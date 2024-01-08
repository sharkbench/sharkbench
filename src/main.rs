extern crate core;

use std::fs;
use clap::Parser;
use docker_stats::DockerStatsReader;
use crate::benchmark::computation::benchmark_computation;
use crate::benchmark::web::benchmark_web;
use crate::utils::docker_runner::run_docker_compose;
use crate::utils::docker_stats;

mod benchmark {
    pub mod computation;
    pub mod web;
    pub mod benchmark;
}

mod utils {
    pub mod docker_runner;
    pub mod docker_stats;
    pub mod http_load_tester;
    pub mod meta_data_parser;
    pub mod percentile;
    pub mod result_writer;
    pub mod serialization;
    pub mod version;
}

/// Benchmarking tool for Sharkbench written in Rust.
///
/// To only run a specific benchmark, use the `--only` flag.
/// Example: `cargo run --release -- --web --only rust/axum-0.7-rust-1.74`
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "blabla")]
struct Args {
    /// Run the computation benchmark
    #[arg(short, long)]
    computing: bool,

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
}

const CONTAINER_NAME: &str = "benchmark";
const WEB_DATASOURCE_DIR: &str = "src/benchmark/web/data";

fn main() {
    let mut args = Args::parse();

    let mut reader = DockerStatsReader::new(CONTAINER_NAME);
    reader.run();

    if let Some(dir) = args.only {
        if args.computing {
            let full_dir = format!("benchmark/computing{}", dir);
            println!(" -> Running only {}", full_dir);
            benchmark_computation(full_dir.as_str(), &mut reader);
        } else if args.web {
            let full_dir = format!("benchmark/web{}", dir);
            println!(" -> Running only {}", full_dir);
            run_docker_compose(WEB_DATASOURCE_DIR, None, || {
                benchmark_web(full_dir.as_str(), &mut reader);
            });
        } else {
            panic!("No benchmark selected");
        }

        reader.stop();
        reader.dispose();
        return;
    } else if let Some(language) = args.lang {
        if args.computing {
            let full_dir = format!("benchmark/computing/{}", language);
            println!(" -> Running only {}", full_dir);
            run_one_language(full_dir.as_str(), &mut reader, benchmark_computation);
        } else if args.web {
            let full_dir = format!("benchmark/web/{}", language);
            println!(" -> Running only {}", full_dir);
            run_docker_compose(WEB_DATASOURCE_DIR, None, || {
                run_one_language(full_dir.as_str(), &mut reader, benchmark_web);
            });
        } else {
            panic!("No benchmark selected");
        }

        reader.stop();
        reader.dispose();
        return;
    }

    if !args.computing && !args.web {
        // By default, run all benchmarks.
        args.computing = true;
        args.web = true;
    }

    if args.computing {
        println!(" -> Running computation benchmarks");
        run_all_languages("benchmark/computation", &mut reader, benchmark_computation);
    } else if args.web {
        println!(" -> Running web benchmarks");
        run_docker_compose(WEB_DATASOURCE_DIR, None, || {
            run_all_languages("benchmark/web", &mut reader, benchmark_web);
        });
    } else {
        panic!("No benchmark selected");
    }
}

fn run_all_languages<F>(dir: &str, reader: &mut DockerStatsReader, run: F)
    where F: Fn(&str, &mut DockerStatsReader) {
    let languages = fs::read_dir(dir).unwrap();
    for language in languages {
        let language = language.unwrap();
        if !language.file_type().unwrap().is_dir() {
            continue;
        }

        run_one_language(language.path().to_str().unwrap(), reader, &run);
    }
}

fn run_one_language<F>(dir: &str, reader: &mut DockerStatsReader, run: F)
    where F: Fn(&str, &mut DockerStatsReader) {
    let versions = fs::read_dir(dir).unwrap();
    for version in versions {
        let version = version.unwrap();
        if !version.file_type().unwrap().is_dir() {
            continue;
        }

        let full_dir = format!("{}", version.path().display());
        run(full_dir.as_str(), reader);
    }
}
