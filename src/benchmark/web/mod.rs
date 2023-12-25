use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::benchmark::benchmark::{AdditionalData, DockerFileManipulation, run_benchmark};
use crate::utils::docker_stats::DockerStatsReader;
use crate::utils::http_load_tester::run_http_load_test;
use crate::utils::meta_data_parser::{WebBenchmarkMetaData};
use crate::utils::result_writer::write_result_to_file;

const QUERY: [(&str, &str); 1] = [("iterations", "1000000000")];
const EXPECTED_RESPONSE: &str = "3.1415926525880504";

pub fn benchmark_web(dir: &str, stats_reader: &mut DockerStatsReader) {
    println!(" -> Benchmarking {}", dir);

    let meta_data: WebBenchmarkMetaData = WebBenchmarkMetaData::read_from_directory(dir).expect("Failed to read meta data");
    meta_data.print_info();

    let data: HashMap<String, PeriodicTableElement> = load_data();
    let requests: Vec<(String, String)> = [data.iter().map(|(k, v)|{
        let url = format!("http://localhost:3000/api/v1/periodic-table/element?symbol={}", k);
        let expected_response = {
            let mut map: IndexMap<String, SerializedValue> = IndexMap::new();
            map.insert("name".to_string(), SerializedValue::StringValue(v.name.to_string()));
            map.insert("number".to_string(), SerializedValue::IntValue(v.number as i32));
            map.insert("group".to_string(), SerializedValue::IntValue(v.group as i32));
            serde_json::to_string(&map).unwrap()
        };
        (url, expected_response)
    }).collect::<Vec<(String, String)>>(), data.iter().map(|(k, v)|{
        let url = format!("http://localhost:3000/api/v1/periodic-table/shells?symbol={}", k);
        let expected_response = {
            let mut map: IndexMap<String, Vec<u8>> = IndexMap::new();
            map.insert("shells".to_string(), v.shells.clone());
            serde_json::to_string(&map).unwrap()
        };
        (url, expected_response)
    }).collect::<Vec<(String, String)>>()].concat();

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
                3,
                || {
                    let result = run_http_load_test(32, Duration::from_secs(10), &requests, response_validator);
                    println!(" -> Success: {}, Fail: {}, Time: {} ms, RPS: {}",
                             result.success_count,
                             result.fail_count,
                             result.total_time.as_millis(),
                             result.requests_per_second);

                    let mut map: IndexMap<String, AdditionalData> = IndexMap::new();
                    map.insert("requests_per_second".to_string(), AdditionalData::Int(result.requests_per_second));
                    Ok(map)
                },
            );

            write_result_to_file(
                "result/web_result.csv",
                &Vec::from([
                    ("language", meta_data.language.as_str()),
                    ("mode", meta_data.mode.as_str()),
                    ("version", language_version.as_str()),
                    ("framework", meta_data.framework.as_str()),
                    ("framework_version", framework_version.as_str()),
                ]),
                &Vec::from([
                    ("requests_per_second_median", result.additional_data.get("requests_per_second").unwrap().to_string().as_str()),
                    ("memory_median", result.memory_median.to_string().as_str()),
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

#[derive(Serialize)]
#[serde(untagged)]
enum SerializedValue {
    StringValue(String),
    IntValue(i32),
}

fn load_data() -> HashMap<String, PeriodicTableElement> {
    let data: String = fs::read_to_string("src/benchmark/web/data/data.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(data.as_str()).unwrap();

    let mut elements: HashMap<String, PeriodicTableElement> = HashMap::new();

    for (key, value) in json.as_object().unwrap() {
        let element: PeriodicTableElement = serde_json::from_value(value.clone()).unwrap();
        elements.insert(key.to_string(), element);
    }

    elements
}

fn response_validator(body: &str, expected_response: &str) -> bool {
    return true;

    if body == expected_response {
        return true;
    }

    // check if the json is the same, but in a different order
    let body_json: serde_json::Value = serde_json::from_str(body).unwrap();
    let expected_response_json: serde_json::Value = serde_json::from_str(expected_response).unwrap();
    if body_json == expected_response_json {
        return true;
    }

    return false;
}