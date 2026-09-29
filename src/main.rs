extern crate core;

use crate::benchmark::computation::{benchmark_computation, count_computation};
use crate::benchmark::web::{benchmark_web, count_web};
use crate::utils::benchmark_limit::BenchmarkLimit;
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

    /// Exit after N benchmarks have been run (skipped benchmarks are not counted).
    /// Useful with `--missing` to let the machine cool down between runs.
    #[arg(long, value_name = "N")]
    limit: Option<usize>,

    /// Do not run any benchmarks, only print how many would be run.
    /// Respects `--missing`, `--lang` and `--only`.
    #[arg(long)]
    count: bool,

    /// Reduce the benchmark time to a minimum to only check if it runs.
    /// No results will be saved.
    #[arg(long)]
    validate: bool,
}

const CONTAINER_NAME: &str = "benchmark";
const WEB_DATASOURCE_DIR: &str = "src/benchmark/web/data";

fn main() {
    let mut args = Args::parse();

    let existing_results: ResultMap = match args.missing {
        true => utils::result_reader::read_existing_result_map(),
        false => ResultMap::default(),
    };

    if args.count {
        println!("{}", count_benchmarks(&args, &existing_results));
        return;
    }

    let mut reader = DockerStatsReader::new();
    reader.run(CONTAINER_NAME);

    let limit = BenchmarkLimit::new(args.limit);

    if let Some(dir) = args.only {
        let (language, variant) = parse_only_dir(&dir);
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
                &limit,
                args.validate,
            );
        } else if args.web {
            let full_dir = format!("benchmark/web/{}", dir);
            println!(" -> Running only {}", full_dir);
            run_docker_compose(WEB_DATASOURCE_DIR, Duration::ZERO, None, false, || {
                benchmark_web(
                    full_dir.as_str(),
                    existing_results
                        .web
                        .get(&language)
                        .and_then(|map| map.get(&variant)),
                    &mut reader,
                    &limit,
                    args.validate,
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
                &limit,
                |dir: &str, existing: Option<&ExistingResult>, reader: &mut DockerStatsReader| {
                    benchmark_computation(dir, existing, reader, &limit, args.validate)
                },
            );
        } else if args.web {
            let full_dir = format!("benchmark/web/{}", language);
            println!(" -> Running only {}", full_dir);
            run_docker_compose(WEB_DATASOURCE_DIR, Duration::ZERO, None, false, || {
                run_one_language(
                    full_dir.as_str(),
                    existing_results.web.get(&language),
                    &mut reader,
                    &limit,
                    |dir: &str,
                     existing: Option<&ExistingResult>,
                     reader: &mut DockerStatsReader| {
                        benchmark_web(dir, existing, reader, &limit, args.validate, args.verbose)
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
            &limit,
            |dir: &str, existing: Option<&ExistingResult>, reader: &mut DockerStatsReader| {
                benchmark_computation(dir, existing, reader, &limit, args.validate)
            },
        );
    }

    if args.web && !limit.reached() {
        println!(" -> Running web benchmarks");
        run_docker_compose(WEB_DATASOURCE_DIR, Duration::ZERO, None, false, || {
            run_all_languages(
                "benchmark/web",
                &existing_results.web,
                &mut reader,
                &limit,
                |dir: &str, existing: Option<&ExistingResult>, reader: &mut DockerStatsReader| {
                    benchmark_web(dir, existing, reader, &limit, args.validate, args.verbose)
                },
            );
        });
    }
}

fn run_all_languages<F>(
    dir: &str,
    skip_existing: &HashMap<String, HashMap<String, ExistingResult>>,
    reader: &mut DockerStatsReader,
    limit: &BenchmarkLimit,
    run: F,
) where
    F: Fn(&str, Option<&ExistingResult>, &mut DockerStatsReader),
{
    for (language, full_dir) in sub_dirs(dir) {
        if limit.reached() {
            break;
        }

        run_one_language(
            full_dir.as_str(),
            skip_existing.get(&language),
            reader,
            limit,
            &run,
        );
    }
}

fn run_one_language<F>(
    dir: &str,
    skip_existing: Option<&HashMap<String, ExistingResult>>,
    reader: &mut DockerStatsReader,
    limit: &BenchmarkLimit,
    run: F,
) where
    F: Fn(&str, Option<&ExistingResult>, &mut DockerStatsReader),
{
    for (variant, full_dir) in variant_dirs(dir) {
        if limit.reached() {
            break;
        }

        let existing_result = skip_existing.and_then(|map| map.get(&variant));

        println!();
        run(&full_dir, existing_result, reader);
    }
}

/// Returns the number of benchmarks that would be run with the given arguments.
fn count_benchmarks(args: &Args, existing_results: &ResultMap) -> usize {
    if let Some(dir) = &args.only {
        let (language, variant) = parse_only_dir(dir);
        return if args.computation {
            count_computation(
                format!("benchmark/computation/{}", dir).as_str(),
                existing_results
                    .computation
                    .get(&language)
                    .and_then(|map| map.get(&variant)),
            )
        } else if args.web {
            count_web(
                format!("benchmark/web/{}", dir).as_str(),
                existing_results
                    .web
                    .get(&language)
                    .and_then(|map| map.get(&variant)),
            )
        } else {
            panic!("No benchmark selected");
        };
    }

    if let Some(language) = &args.lang {
        return if args.computation {
            count_one_language(
                format!("benchmark/computation/{}", language).as_str(),
                existing_results.computation.get(language),
                count_computation,
            )
        } else if args.web {
            count_one_language(
                format!("benchmark/web/{}", language).as_str(),
                existing_results.web.get(language),
                count_web,
            )
        } else {
            panic!("No benchmark selected");
        };
    }

    // By default, count all benchmarks.
    let all = !args.computation && !args.web;
    let mut count = 0;
    if args.computation || all {
        count += count_all_languages(
            "benchmark/computation",
            &existing_results.computation,
            count_computation,
        );
    }
    if args.web || all {
        count += count_all_languages("benchmark/web", &existing_results.web, count_web);
    }
    count
}

fn count_all_languages(
    dir: &str,
    skip_existing: &HashMap<String, HashMap<String, ExistingResult>>,
    count: fn(&str, Option<&ExistingResult>) -> usize,
) -> usize {
    sub_dirs(dir)
        .iter()
        .map(|(language, full_dir)| {
            count_one_language(full_dir.as_str(), skip_existing.get(language), count)
        })
        .sum()
}

fn count_one_language(
    dir: &str,
    skip_existing: Option<&HashMap<String, ExistingResult>>,
    count: fn(&str, Option<&ExistingResult>) -> usize,
) -> usize {
    variant_dirs(dir)
        .iter()
        .map(|(variant, full_dir)| {
            count(
                full_dir.as_str(),
                skip_existing.and_then(|map| map.get(variant)),
            )
        })
        .sum()
}

/// Parses the `--only` argument into (language, variant).
fn parse_only_dir(dir: &str) -> (String, String) {
    let parts: Vec<&str> = dir.split('/').collect();
    if parts.len() != 2 {
        panic!("Invalid directory format. Expected <language>/<variant>");
    }
    (parts[0].to_string(), parts[1].to_string())
}

/// Returns (name, full_dir) of each benchmark variant directory of a language,
/// excluding the common directory.
fn variant_dirs(dir: &str) -> Vec<(String, String)> {
    sub_dirs(dir)
        .into_iter()
        .filter(|(name, _)| name != utils::copy_files::COMMON_DIR)
        .collect()
}

/// Returns (name, full_dir) of each subdirectory in `dir`.
fn sub_dirs(dir: &str) -> Vec<(String, String)> {
    fs::read_dir(dir)
        .expect(&format!("Could not read directory {}", dir))
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| {
            let name = entry.file_name().to_str().unwrap().to_owned();
            // Always use forward slashes so the path stored in the result is platform independent
            let full_dir = format!("{}/{}", dir, name);
            (name, full_dir)
        })
        .collect()
}
