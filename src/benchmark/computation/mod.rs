use crate::benchmark::benchmark::{run_benchmark, IterationResult};
use crate::benchmark::runner::RunConfig;
use crate::benchmark::task::ComputationTask;
use crate::utils::copy_files;
use crate::utils::docker_runner::StartMode;
use crate::utils::docker_stats::DockerStatsReader;
use crate::utils::result_writer::write_result_to_file;
use crate::utils::version_migrator::VersionMigrator;
use crate::utils::work_dir;
use indexmap::IndexMap;
use std::time::Duration;

/// The program is driven over stdin/stdout, so no port is published.
const COMPOSE_FILE: &str = r#"
services:
  benchmark:
    build: .
    container_name: benchmark
    deploy:
      resources:
        limits:
          cpus: "1.0"
"#;

/// Number of Leibniz iterations sent to the program on each run
const ITERATIONS: &str = "1000000000";
const EXPECTED_RESPONSE: &str = "3.1415926525880504;785398157.7092886;0.7853981633136793";
const DEFAULT_RUNS: usize = 15;

pub fn benchmark_computation(
    task: &ComputationTask,
    stats_reader: &mut DockerStatsReader,
    config: &RunConfig,
) {
    let meta_data = &task.meta_data;

    let runs = match config.validate {
        true => 1,
        false => match meta_data.runs {
            Some(runs) => {
                println!(" -> Running {runs} times instead of default = {DEFAULT_RUNS}");
                runs
            }
            None => DEFAULT_RUNS,
        },
    };

    let src_dir = task.dir.path.as_str();
    let dir = work_dir::prepare_work_dir(src_dir);

    if let Some(copy_files) = &meta_data.copy {
        copy_files::copy_files(src_dir, dir, &copy_files);
    }

    let version_migrations: Vec<VersionMigrator> = match meta_data.language_version.len() {
        1 => vec![],
        _ => vec![VersionMigrator::new(
            dir,
            meta_data.language_version_regex.clone(),
            meta_data.language_version[0].clone(),
            task.language_version.clone(),
        )],
    };
    let result = run_benchmark(
        dir,
        COMPOSE_FILE,
        stats_reader,
        &version_migrations,
        match config.validate {
            true => 0,
            false => match meta_data.extended_warmup {
                true => 3,
                false => 1,
            },
        },
        runs,
        StartMode::Attached,
        |container| {
            let container = container.expect("Attached start mode provides a container");
            container.write_line(ITERATIONS)?;
            let body = container.read_line(Duration::from_secs(600))?;
            if !body.contains(EXPECTED_RESPONSE) {
                return Err(Box::from(format!(
                    "Invalid response: {} (expected: {})",
                    body, EXPECTED_RESPONSE
                )));
            }

            Ok(IterationResult {
                additional_data: IndexMap::new(),
                debugging_data: IndexMap::new(),
            })
        },
    );

    work_dir::remove_work_dir();

    if config.validate {
        return;
    }

    write_result_to_file(
        "result/computation_result.csv",
        &Vec::from([
            ("language", meta_data.language.as_str()),
            ("mode", meta_data.mode.as_str()),
            ("version", task.language_version.as_str()),
            ("path", task.dir.result_path().as_str()),
        ]),
        &Vec::from([
            ("time_median", result.time_median.to_string().as_str()),
            ("memory_median", result.memory_median.to_string().as_str()),
            ("build_time", result.build_time.to_string().as_str()),
        ]),
        take_lower_time_median,
    )
    .expect("Failed to write result to file");
}

fn take_lower_time_median<'a>(
    old_values: &'a [&'a str],
    new_values: &'a [&'a str],
) -> &'a [&'a str] {
    if old_values[0].parse::<i32>().unwrap() < new_values[0].parse::<i32>().unwrap() {
        println!(
            " -> Keeping old values (time_median: {} < {})",
            old_values[0], new_values[0]
        );
        old_values
    } else {
        new_values
    }
}
