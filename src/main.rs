extern crate core;

use crate::benchmark::computation::benchmark_computation;
use crate::benchmark::web::benchmark_web;
use crate::utils::docker_runner::run_docker_compose;
use crate::utils::docker_stats;
use crate::utils::result_reader::{ExistingResult, ResultMap};
use clap::Parser;
use docker_stats::DockerStatsReader;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;

mod benchmark;
mod utils;

/// Benchmarking tool for Sharkbench written in Rust.
///
/// To only run a specific benchmark, use the `--only` flag.
/// Example: `cargo run --release -- --web --only rust/axum-0.7-rust-1.74`
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Run the computation benchmark
    #[arg(short, long)]
    computation: bool,

    /// Run the web benchmark
    #[arg(short, long)]
    web: bool,

    /// Only runs the benchmark specified of a specific language
    /// Must be used with `--web` or `--computing`
    #[arg(short, long, value_name = "LANG")]
    lang: Option<String>,

    /// Only runs the benchmark specified in the specified directory
    /// Must be used with `--web` or `--computing`
    #[arg(long, value_name = "DIR")]
    only: Option<String>,

    /// Print more information
    #[arg(short, long)]
    verbose: bool,

    /// Only run missing benchmarks
    #[arg(long)]
    missing: bool,
}

const CONTAINER_NAME: &str = "benchmark";
const WEB_DATASOURCE_DIR: &str = "src/benchmark/web/data";

fn main() {
    let mut args = Args::parse();

    let mut reader = DockerStatsReader::new();
    reader.run(CONTAINER_NAME);

    let existing_results: ResultMap = match args.missing {
        true => utils::result_reader::read_existing_result_map(),
        false => ResultMap::default(),
    };

    if let Some(dir) = args.only {
        let (language, variant) = {
            let parts: Vec<&str> = dir.split('/').collect();
            if parts.len() != 2 {
                panic!("Invalid directory format. Expected <language>/<variant>");
            }
            (parts[0].to_string(), parts[1].to_string())
        };
        if args.computation {
            let full_dir = format!("benchmark/computation/{}", dir);
            println!(" -> Running only {}", full_dir);
            benchmark_computation(
                full_dir.as_str(),
                existing_results
                    .computation
                    .get(&language)
                    .and_then(|map| map.get(&variant)),
                &mut reader,
            );
        } else if args.web {
            let full_dir = format!("benchmark/web/{}", dir);
            println!(" -> Running only {}", full_dir);
            run_docker_compose(WEB_DATASOURCE_DIR, Duration::ZERO, None, || {
                benchmark_web(
                    full_dir.as_str(),
                    existing_results
                        .web
                        .get(&language)
                        .and_then(|map| map.get(&variant)),
                    &mut reader,
                    args.verbose,
                );
            });
        } else {
            panic!("No benchmark selected");
        }

        reader.stop();
        reader.dispose();
        return;
    }

    if let Some(language) = args.lang {
        if args.computation {
            let full_dir = format!("benchmark/computation/{}", language);
            println!(" -> Running only {}", full_dir);
            run_one_language(
                full_dir.as_str(),
                existing_results.computation.get(&language),
                &mut reader,
                benchmark_computation,
            );
        } else if args.web {
            let full_dir = format!("benchmark/web/{}", language);
            println!(" -> Running only {}", full_dir);
            run_docker_compose(WEB_DATASOURCE_DIR, Duration::ZERO, None, || {
                run_one_language(
                    full_dir.as_str(),
                    existing_results.web.get(&language),
                    &mut reader,
                    |dir: &str,
                     existing: Option<&ExistingResult>,
                     reader: &mut DockerStatsReader| {
                        benchmark_web(dir, existing, reader, args.verbose)
                    },
                );
            });
        } else {
            panic!("No benchmark selected");
        }

        reader.stop();
        reader.dispose();
        return;
    }

    if !args.computation && !args.web {
        // By default, run all benchmarks.
        args.computation = true;
        args.web = true;
    }

    if args.computation {
        println!(" -> Running computation benchmarks");
        run_all_languages(
            "benchmark/computation",
            &existing_results.computation,
            &mut reader,
            benchmark_computation,
        );
    }

    if args.web {
        println!(" -> Running web benchmarks");
        run_docker_compose(WEB_DATASOURCE_DIR, Duration::ZERO, None, || {
            run_all_languages(
                "benchmark/web",
                &existing_results.web,
                &mut reader,
                |dir: &str, existing: Option<&ExistingResult>, reader: &mut DockerStatsReader| {
                    benchmark_web(dir, existing, reader, args.verbose)
                },
            );
        });
    }
}

fn run_all_languages<F>(
    dir: &str,
    skip_existing: &HashMap<String, HashMap<String, ExistingResult>>,
    reader: &mut DockerStatsReader,
    run: F,
) where
    F: Fn(&str, Option<&ExistingResult>, &mut DockerStatsReader),
{
    let languages = fs::read_dir(dir).unwrap();
    for language_folder in languages {
        let language_folder = language_folder.unwrap();
        if !language_folder.file_type().unwrap().is_dir() {
            continue;
        }

        run_one_language(
            language_folder.path().to_str().unwrap(),
            skip_existing.get(language_folder.file_name().to_str().unwrap()),
            reader,
            &run,
        );
    }
}

fn run_one_language<F>(
    dir: &str,
    skip_existing: Option<&HashMap<String, ExistingResult>>,
    reader: &mut DockerStatsReader,
    run: F,
) where
    F: Fn(&str, Option<&ExistingResult>, &mut DockerStatsReader),
{
    let variants = fs::read_dir(dir).expect(&format!("Could not read directory {}", dir));
    for variant_folder in variants {
        let variant_folder = variant_folder.unwrap();
        if !variant_folder.file_type().unwrap().is_dir() {
            // Only run on directories
            continue;
        }

        let directory_name = variant_folder.file_name().to_str().unwrap().to_owned();

        if directory_name == utils::copy_files::COMMON_DIR {
            // Skip the common directory
            continue;
        }

        let full_dir = format!("{}", variant_folder.path().display());

        let existing_result = skip_existing.and_then(|map| map.get(&directory_name));

        println!();
        run(&full_dir, existing_result, reader);
    }
}
