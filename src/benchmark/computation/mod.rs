use indexmap::IndexMap;
use crate::benchmark::benchmark::{DockerFileManipulation, IterationResult, run_benchmark};
use crate::utils::docker_stats::DockerStatsReader;
use crate::utils::meta_data_parser::BenchmarkMetaData;
use crate::utils::result_writer::write_result_to_file;

const QUERY: [(&str, &str); 1] = [("iterations", "1000000000")];
const EXPECTED_RESPONSE: &str = "3.1415926525880504";

pub fn benchmark_computation(dir: &str, stats_reader: &mut DockerStatsReader) {
    println!(" -> Benchmarking {}", dir);

    let meta_data: BenchmarkMetaData = BenchmarkMetaData::read_from_directory(dir).expect("Failed to read meta data");
    meta_data.print_info();

    for language_version in &meta_data.language_version {
        let docker_file_manipulation: Option<DockerFileManipulation> = match meta_data.language_version.len() {
            1 => None,
            _ => Some(DockerFileManipulation {
                initial_from_version: meta_data.language_version[0].clone(),
                new_from_version: language_version.clone(),
            }),
        };
        let result = run_benchmark(
            dir,
            stats_reader,
            &docker_file_manipulation,
            3,
            || {
                let client = reqwest::blocking::Client::new();
                let response = match client.get("http://localhost:3000").query(&QUERY).send() {
                    Ok(response) => Ok(response),
                    Err(e) => Err(e.to_string()),
                }?;
                let body = response.text()?;
                if body != EXPECTED_RESPONSE {
                    return Err(Box::from(format!("Invalid response: {} (expected: {})", body, EXPECTED_RESPONSE)));
                }

                Ok(IterationResult {
                    additional_data: IndexMap::new(),
                    debugging_data: IndexMap::new(),
                })
            },
        );

        write_result_to_file(
            "result/computation_result.csv",
            &Vec::from([
                ("language", meta_data.language.as_str()),
                ("mode", meta_data.mode.as_str()),
                ("version", language_version.as_str()),
            ]),
            &Vec::from([
                ("time_median", result.time_median.to_string().as_str()),
                ("memory_median", result.memory_median.to_string().as_str()),
            ]),
        ).expect("Failed to write result to file");
    }
}
