use crate::benchmark::benchmark::{run_benchmark, IterationResult};
use crate::utils::copy_files;
use crate::utils::docker_stats::DockerStatsReader;
use crate::utils::meta_data_parser::BenchmarkMetaData;
use crate::utils::result_reader::ExistingResult;
use crate::utils::result_writer::write_result_to_file;
use crate::utils::version_migrator::VersionMigrator;
use indexmap::IndexMap;
use std::time::Duration;

const QUERY: [(&str, &str); 1] = [("iterations", "1000000000")];
const EXPECTED_RESPONSE: &str = "3.1415926525880504;785398157.7092886;0.7853981633136793";
const DEFAULT_RUNS: usize = 15;

pub fn benchmark_computation(
    dir: &str,
    existing: Option<&ExistingResult>,
    stats_reader: &mut DockerStatsReader,
    validate: bool,
) {
    let meta_data: BenchmarkMetaData = BenchmarkMetaData::read_from_directory(dir)
        .expect(&format!("Failed to read meta data: {dir}"));

    // Early check if all existing results are in metadata to avoid printing metadata info
    if let Some(existing) = existing {
        if meta_data
            .language_version
            .iter()
            .all(|lang_version| existing.language_versions.contains(lang_version))
        {
            println!(" -> Skipping {dir}");
            return;
        }
    }

    println!(" -> Benchmarking {}", dir);
    meta_data.print_info();

    let runs = match validate {
        true => 1,
        false => match meta_data.runs {
            Some(runs) => {
                println!(" -> Running {runs} times instead of default = {DEFAULT_RUNS}");
                runs
            }
            None => DEFAULT_RUNS,
        },
    };

    for language_version in &meta_data.language_version {
        if let Some(existing) = existing {
            if existing.language_versions.contains(language_version) {
                println!(
                    " -> Skipping {} v{} (already exists)",
                    meta_data.mode, language_version
                );
                continue;
            }
        }

        if let Some(copy_files) = &meta_data.copy {
            copy_files::copy_files(dir, &copy_files);
        }

        let mut version_migrations: Vec<VersionMigrator> = match meta_data.language_version.len() {
            1 => vec![],
            _ => vec![VersionMigrator::new(
                dir,
                meta_data.language_version_regex.clone(),
                meta_data.language_version[0].clone(),
                language_version.clone(),
            )],
        };
        let result = run_benchmark(
            dir,
            stats_reader,
            version_migrations.iter_mut().collect(),
            match validate {
                true => 0,
                false => match meta_data.extended_warmup {
                    true => 3,
                    false => 1,
                },
            },
            runs,
            || {
                let client = reqwest::blocking::Client::new();
                let response = match client
                    .get("http://localhost:3000")
                    .query(&QUERY)
                    .timeout(Duration::from_secs(600))
                    .send()
                {
                    Ok(response) => Ok(response),
                    Err(e) => Err(e.to_string()),
                }?;
                let body = response.text()?;
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

        if let Some(copy_files) = &meta_data.copy {
            copy_files::delete_copied_files(dir, &copy_files);
        }

        if validate {
            continue;
        }

        write_result_to_file(
            "result/computation_result.csv",
            &Vec::from([
                ("language", meta_data.language.as_str()),
                ("mode", meta_data.mode.as_str()),
                ("version", language_version.as_str()),
                ("path", dir.replace("benchmark/computation/", "").as_str()),
            ]),
            &Vec::from([
                ("time_median", result.time_median.to_string().as_str()),
                ("memory_median", result.memory_median.to_string().as_str()),
            ]),
            take_lower_time_median,
        )
        .expect("Failed to write result to file");
    }
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
