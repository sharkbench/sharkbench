extern crate core;

use crate::benchmark::planner::{plan_tasks, Selection, TaskFilter};
use crate::benchmark::pruner::prune_results;
use crate::benchmark::runner::{run_tasks, RunConfig};
use crate::utils::docker_runner::CONTAINER_NAME;
use crate::utils::docker_stats;
use clap::Parser;
use docker_stats::DockerStatsReader;

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

    /// Only run the last (highest) version combination of each benchmark
    #[arg(long)]
    only_latest: bool,

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
    /// Respects `--missing`, `--only-latest`, `--lang` and `--only`.
    /// With `--prune`, only print the rows that would be removed.
    #[arg(long)]
    count: bool,

    /// Do not run any benchmarks, instead remove result rows
    /// not belonging to any benchmark (version) anymore.
    /// Respects `--computation`, `--web`, `--lang` and `--only`.
    #[arg(long, conflicts_with_all = ["only_latest", "missing", "limit", "validate"])]
    prune: bool,

    /// Reduce the benchmark time to a minimum to only check if it runs.
    /// No results will be saved.
    #[arg(long)]
    validate: bool,
}

fn main() {
    let args = Args::parse();

    let selection = match (args.only, args.lang) {
        (Some(dir), _) => Selection::Only(dir),
        (None, Some(language)) => Selection::Language(language),
        (None, None) => Selection::All,
    };

    let (computation, web) = match (args.computation, args.web, &selection) {
        // By default, run all benchmarks.
        (false, false, Selection::All) => (true, true),
        (false, false, _) => panic!("No benchmark selected"),
        (computation, web, _) => (computation, web),
    };

    let filter = TaskFilter {
        computation,
        web,
        selection,
        only_latest: args.only_latest,
    };
    let mut tasks = plan_tasks(&filter);

    if args.prune {
        let removed = prune_results(&tasks, &filter, args.count);
        if removed == 0 {
            println!(" -> Nothing to prune");
        }
        return;
    }

    let mut skipped = 0;
    if args.missing {
        let existing_results = utils::result_reader::read_existing_result_map();
        let planned = tasks.len();
        tasks.retain(|task| !task.has_result(&existing_results));
        skipped = planned - tasks.len();
    }

    if args.count {
        println!("{}", tasks.len());
        return;
    }

    if skipped > 0 {
        println!(" -> Skipping {skipped} benchmarks with existing results");
    }

    if let Some(limit) = args.limit {
        if tasks.len() > limit {
            println!(" -> Limiting to {limit} of {} benchmarks", tasks.len());
            tasks.truncate(limit);
        }
    }

    if tasks.is_empty() {
        println!(" -> Nothing to run");
        return;
    }

    println!(" -> Running {} benchmarks:", tasks.len());
    for task in &tasks {
        println!("    - {task}");
    }

    let mut reader = DockerStatsReader::new();
    reader.run(CONTAINER_NAME);

    run_tasks(
        tasks,
        &mut reader,
        &RunConfig {
            validate: args.validate,
            verbose: args.verbose,
        },
    );

    reader.stop();
    reader.dispose();
}
