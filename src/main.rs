extern crate core;

use crate::benchmark::computation::{benchmark_computation, count_computation};
use crate::benchmark::web::{benchmark_web, count_web};
use crate::utils::benchmark_limit::BenchmarkLimit;
use crate::utils::docker_runner::{run_docker_compose, StartMode, CONTAINER_NAME};
use crate::utils::docker_stats;
use crate::utils::result_reader::{ExistingResult, ResultMap};
use clap::Parser;
use docker_stats::DockerStatsReader;
use std::collections::HashMap;
use std::fs;

mod benchmark;
mod utils;

/// Benchmarking tool for Sharkbench written in Rust.
///
/// To only run a specific benchmark, use the `--only` flag.
/// Example: `cargo run --release -- --web --only rust/axum-0.7-rust-1.74`
/// The version can be omitted to run all matching variants: `--only rust/axum`
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
        if args.computation {
            let (language, variants) = resolve_only_dir("benchmark/computation", &dir);
            print_only_dirs(&variants);
            run_variants(
                variants,
                existing_results.computation.get(&language),
                &mut reader,
                &limit,
                |dir: &str, existing: Option<&ExistingResult>, reader: &mut DockerStatsReader| {
                    benchmark_computation(dir, existing, reader, &limit, args.validate)
                },
            );
        } else if args.web {
            let (language, variants) = resolve_only_dir("benchmark/web", &dir);
            print_only_dirs(&variants);
            run_docker_compose(
                WEB_DATASOURCE_DIR,
                StartMode::Detached { ready_port: None },
                None,
                false,
                |_| {
                    run_variants(
                        variants,
                        existing_results.web.get(&language),
                        &mut reader,
                        &limit,
                        |dir: &str,
                         existing: Option<&ExistingResult>,
                         reader: &mut DockerStatsReader| {
                            benchmark_web(
                                dir,
                                existing,
                                reader,
                                &limit,
                                args.validate,
                                args.verbose,
                            )
                        },
                    );
                },
            );
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
            run_docker_compose(
                WEB_DATASOURCE_DIR,
                StartMode::Detached { ready_port: None },
                None,
                false,
                |_| {
                    run_one_language(
                        full_dir.as_str(),
                        existing_results.web.get(&language),
                        &mut reader,
                        &limit,
                        |dir: &str,
                         existing: Option<&ExistingResult>,
                         reader: &mut DockerStatsReader| {
                            benchmark_web(
                                dir,
                                existing,
                                reader,
                                &limit,
                                args.validate,
                                args.verbose,
                            )
                        },
                    );
                },
            );
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
        run_docker_compose(
            WEB_DATASOURCE_DIR,
            StartMode::Detached { ready_port: None },
            None,
            false,
            |_| {
                run_all_languages(
                    "benchmark/web",
                    &existing_results.web,
                    &mut reader,
                    &limit,
                    |dir: &str,
                     existing: Option<&ExistingResult>,
                     reader: &mut DockerStatsReader| {
                        benchmark_web(dir, existing, reader, &limit, args.validate, args.verbose)
                    },
                );
            },
        );
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
    for language in sub_dirs(dir) {
        if limit.reached() {
            break;
        }

        run_one_language(
            language.full_dir.as_str(),
            skip_existing.get(&language.name),
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
    run_variants(variant_dirs(dir), skip_existing, reader, limit, run);
}

/// Runs the given variants of a single language.
fn run_variants<F>(
    variants: Vec<SubDir>,
    skip_existing: Option<&HashMap<String, ExistingResult>>,
    reader: &mut DockerStatsReader,
    limit: &BenchmarkLimit,
    run: F,
) where
    F: Fn(&str, Option<&ExistingResult>, &mut DockerStatsReader),
{
    for variant in variants {
        if limit.reached() {
            break;
        }

        let existing_result = skip_existing.and_then(|map| map.get(&variant.name));

        println!();
        run(&variant.full_dir, existing_result, reader);
    }
}

/// Returns the number of benchmarks that would be run with the given arguments.
fn count_benchmarks(args: &Args, existing_results: &ResultMap) -> usize {
    if let Some(dir) = &args.only {
        return if args.computation {
            let (language, variants) = resolve_only_dir("benchmark/computation", dir);
            count_variants(
                &variants,
                existing_results.computation.get(&language),
                count_computation,
            )
        } else if args.web {
            let (language, variants) = resolve_only_dir("benchmark/web", dir);
            count_variants(&variants, existing_results.web.get(&language), count_web)
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
        .map(|language| {
            count_one_language(
                language.full_dir.as_str(),
                skip_existing.get(&language.name),
                count,
            )
        })
        .sum()
}

fn count_one_language(
    dir: &str,
    skip_existing: Option<&HashMap<String, ExistingResult>>,
    count: fn(&str, Option<&ExistingResult>) -> usize,
) -> usize {
    count_variants(&variant_dirs(dir), skip_existing, count)
}

fn count_variants(
    variants: &[SubDir],
    skip_existing: Option<&HashMap<String, ExistingResult>>,
    count: fn(&str, Option<&ExistingResult>) -> usize,
) -> usize {
    variants
        .iter()
        .map(|variant| {
            count(
                variant.full_dir.as_str(),
                skip_existing.and_then(|map| map.get(&variant.name)),
            )
        })
        .sum()
}

/// Resolves the `--only` argument (`<language>/<variant>`) inside `base_dir`
/// into (language, variants).
///
/// An exact directory match is preferred.
/// Otherwise, the variant is treated as an unversioned prefix,
/// e.g. `rust/rama` matches `rust/rama-0.4-rust-1.98.1`.
fn resolve_only_dir(base_dir: &str, dir: &str) -> (String, Vec<SubDir>) {
    let parts: Vec<&str> = dir.split('/').collect();
    if parts.len() != 2 {
        panic!("Invalid directory format. Expected <language>/<variant>");
    }
    let (language, query) = (parts[0], parts[1]);

    let available = variant_dirs(format!("{}/{}", base_dir, language).as_str());
    let variants = match_variants(&available, query);
    if variants.is_empty() {
        let names: Vec<&String> = available.iter().map(|variant| &variant.name).collect();
        panic!(
            "No benchmark found for {}. Available in {}: {:?}",
            dir, language, names
        );
    }

    (language.to_string(), variants)
}

fn print_only_dirs(variants: &[SubDir]) {
    for variant in variants {
        println!(" -> Running only {}", variant.full_dir);
    }
}

/// Returns the variants matching `query`:
/// the exact match if there is one, otherwise all variants starting with `<query>-`.
fn match_variants(available: &[SubDir], query: &str) -> Vec<SubDir> {
    if let Some(exact) = available.iter().find(|variant| variant.name == query) {
        return vec![exact.clone()];
    }

    let prefix = format!("{}-", query);
    available
        .iter()
        .filter(|variant| variant.name.starts_with(&prefix))
        .cloned()
        .collect()
}

/// A subdirectory of the benchmark tree, i.e. a language or a variant.
#[derive(Debug, Clone)]
struct SubDir {
    /// Directory name, e.g. `axum-0.8.9-rust-1.98.1`
    name: String,

    /// Full path, e.g. `benchmark/web/rust/axum-0.8.9-rust-1.98.1`
    full_dir: String,
}

/// Returns each benchmark variant directory of a language,
/// excluding the common directory.
fn variant_dirs(dir: &str) -> Vec<SubDir> {
    sub_dirs(dir)
        .into_iter()
        .filter(|variant| variant.name != utils::copy_files::COMMON_DIR)
        .collect()
}

/// Returns each subdirectory in `dir`, sorted by name.
fn sub_dirs(dir: &str) -> Vec<SubDir> {
    let mut dirs: Vec<SubDir> = fs::read_dir(dir)
        .expect(&format!("Could not read directory {}", dir))
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| {
            let name = entry.file_name().to_str().unwrap().to_owned();
            // Always use forward slashes so the path stored in the result is platform independent
            let full_dir = format!("{}/{}", dir, name);
            SubDir { name, full_dir }
        })
        .collect();
    // fs::read_dir does not guarantee any order
    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variants(names: &[&str]) -> Vec<SubDir> {
        names
            .iter()
            .map(|name| SubDir {
                name: name.to_string(),
                full_dir: format!("benchmark/web/rust/{}", name),
            })
            .collect()
    }

    fn names(variants: Vec<SubDir>) -> Vec<String> {
        variants.into_iter().map(|variant| variant.name).collect()
    }

    #[test]
    fn should_prefer_exact_match() {
        let available = variants(&["ktor-3", "ktor-3-cio-native"]);
        assert_eq!(names(match_variants(&available, "ktor-3")), vec!["ktor-3"]);
    }

    #[test]
    fn should_match_unversioned() {
        let available = variants(&["rama-0.4-rust-1.98.1", "axum-0.8.9-rust-1.98.1"]);
        assert_eq!(
            names(match_variants(&available, "rama")),
            vec!["rama-0.4-rust-1.98.1"]
        );
    }

    #[test]
    fn should_match_multiple() {
        let available = variants(&["vertx-4-semeru-11", "vertx-4-semeru-21", "vertxx-1"]);
        assert_eq!(
            names(match_variants(&available, "vertx")),
            vec!["vertx-4-semeru-11", "vertx-4-semeru-21"]
        );
    }

    #[test]
    fn should_not_match_partial_name() {
        let available = variants(&["express-5-bun-1"]);
        assert!(match_variants(&available, "bun").is_empty());
        assert!(match_variants(&available, "expr").is_empty());
    }
}
