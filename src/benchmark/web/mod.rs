use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use indexmap::IndexMap;
use serde::{Deserialize};
use crate::benchmark::benchmark::{AdditionalData, DockerFileManipulation, IterationResult, run_benchmark};
use crate::utils::docker_stats::DockerStatsReader;
use crate::utils::http_load_tester::run_http_load_test;
use crate::utils::meta_data_parser::{WebBenchmarkMetaData};
use crate::utils::result_writer::write_result_to_file;
use crate::utils::serialization::SerializedValue;

const DEFAULT_CONCURRENCY: usize = 32;

pub fn benchmark_web(
    dir: &str,
    stats_reader: &mut DockerStatsReader,
    verbose: bool,
) {
    println!(" -> Benchmarking {}", dir);

    let meta_data: WebBenchmarkMetaData = WebBenchmarkMetaData::read_from_directory(dir).expect("Failed to read meta data");
    meta_data.print_info();

    let data: HashMap<String, PeriodicTableElement> = load_data();
    let requests: Vec<(String, HashMap<String, SerializedValue>)> = [data.iter().map(|(k, v)|{
        let url = format!("http://localhost:3000/api/v1/periodic-table/element?symbol={}", k);
        let expected_response = HashMap::from([
            ("name".to_string(), SerializedValue::StringValue(v.name.to_string())),
            ("number".to_string(), SerializedValue::IntValue(v.number as i32)),
            ("group".to_string(), SerializedValue::IntValue(v.group as i32)),
        ]);
        (url, expected_response)
    }).collect::<Vec<(String, HashMap<String, SerializedValue>)>>(), data.iter().map(|(k, v)|{
        let url = format!("http://localhost:3000/api/v1/periodic-table/shells?symbol={}", k);
        let expected_response = HashMap::from([
            ("shells".to_string(), SerializedValue::IntListValue(v.shells.iter().map(|v| *v as i32).collect::<Vec<i32>>())),
        ]);
        (url, expected_response)
    }).collect::<Vec<(String, HashMap<String, SerializedValue>)>>()].concat();

    let concurrency = match meta_data.concurrency {
        Some(concurrency) => {
            println!(" -> Using concurrency = {} instead of default = {}", concurrency, DEFAULT_CONCURRENCY);
            concurrency
        },
        None => DEFAULT_CONCURRENCY,
    };

    for language_version in &meta_data.language_version {
        for framework_version in &meta_data.framework_version {
            let docker_file_manipulation: Option<DockerFileManipulation> = match (meta_data.language_version.len(), meta_data.framework_version.len()) {
                (1, 1) => None,
                _ => Some(DockerFileManipulation {
                    initial_from_version: meta_data.language_version[0].clone(),
                    new_from_version: language_version.clone(),
                }),
            };

            let result = run_benchmark(
                dir,
                stats_reader,
                &docker_file_manipulation,
                match meta_data.extended_warmup {
                    true => 3,
                    false => 1,
                },
                5,
                || {
                    let result = run_http_load_test(
                        concurrency,
                        Duration::from_secs(15),
                        &requests,
                        response_validator,
                        verbose,
                    );

                    let mut additional_data: IndexMap<String, AdditionalData> = IndexMap::new();
                    additional_data.insert("rps_median".to_string(), AdditionalData::Int(result.rps_median));
                    additional_data.insert("rps_p99".to_string(), AdditionalData::Int(result.rps_p99));
                    additional_data.insert("latency_median".to_string(), AdditionalData::Int(result.latency_median.as_micros() as i32));
                    additional_data.insert("latency_p99".to_string(), AdditionalData::Int(result.latency_p99.as_micros() as i32));
                    additional_data.insert("errors".to_string(), AdditionalData::Int(result.fail_count));

                    let mut debugging_data: IndexMap<String, AdditionalData> = IndexMap::new();
                    debugging_data.insert("success".to_string(), AdditionalData::Int(result.success_count));
                    debugging_data.insert("time".to_string(), AdditionalData::Int(result.total_time.as_millis() as i32));

                    Ok(IterationResult {
                        additional_data,
                        debugging_data,
                    })
                },
            );

            write_result_to_file(
                "result/web_result.csv",
                &Vec::from([
                    ("language", meta_data.language.as_str()),
                    ("mode", meta_data.mode.as_str()),
                    ("version", language_version.as_str()),
                    ("framework", meta_data.framework.as_str()),
                    ("framework_website", meta_data.framework_website.as_str()),
                    ("framework_flavor", meta_data.framework_flavor.as_str()),
                    ("framework_version", framework_version.as_str()),
                    ("concurrency", concurrency.to_string().as_str()),
                    ("path", dir.replace("benchmark/web/", "").as_str()),
                ]),
                &Vec::from([
                    ("rps_median", result.additional_data.get("rps_median").unwrap().to_string().as_str()),
                    ("rps_p99", result.additional_data.get("rps_p99").unwrap().to_string().as_str()),
                    ("latency_median", result.additional_data.get("latency_median").unwrap().to_string().as_str()),
                    ("latency_p99", result.additional_data.get("latency_p99").unwrap().to_string().as_str()),
                    ("memory_median", result.memory_median.to_string().as_str()),
                    ("memory_p99", result.memory_p99.to_string().as_str()),
                    ("errors", result.additional_data.get("errors").unwrap().to_string().as_str()),
                ]),
            ).expect("Failed to write result to file");
        }
    }
}

#[derive(Deserialize)]
struct PeriodicTableElement {
    name: String,
    number: u8,
    group: u8,
    shells: Vec<u8>,
}

fn load_data() -> HashMap<String, PeriodicTableElement> {
    let data: String = fs::read_to_string("src/benchmark/web/data/static/data.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(data.as_str()).unwrap();

    let mut elements: HashMap<String, PeriodicTableElement> = HashMap::new();

    for (key, value) in json.as_object().unwrap() {
        let element: PeriodicTableElement = serde_json::from_value(value.clone()).unwrap();
        elements.insert(key.to_string(), element);
    }

    elements
}

fn response_validator(body: &str, expected_response: &HashMap<String, SerializedValue>) -> bool {
    let json: serde_json::Value = {
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(json) => json,
            Err(_) => return false,
        }
    };

    for (key, value) in expected_response {
        let actual_value = json.get(key).unwrap();
        match value {
            SerializedValue::StringValue(v) => {
                if actual_value.as_str().unwrap() != v {
                    return false;
                }
            }
            SerializedValue::IntValue(v) => {
                if actual_value.as_i64().unwrap() != *v as i64 {
                    return false;
                }
            }
            SerializedValue::IntListValue(v) => {
                let actual_list = actual_value.as_array().unwrap();
                if actual_list.len() != v.len() {
                    return false;
                }
                for (i, actual_value) in actual_list.iter().enumerate() {
                    if actual_value.as_i64().unwrap() != v[i] as i64 {
                        return false;
                    }
                }
            }
        }
    }

    true
}
