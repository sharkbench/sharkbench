use std::time::Duration;
use std::{fs, thread};
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use indexmap::IndexMap;
use crate::utils::docker_runner::run_docker_compose;

const COMPOSE_FILE: &str = r#"
version: "3"
services:
  benchmark:
    build: .
    container_name: benchmark
    ports:
      - "3000:3000"
    sysctls:
      - net.ipv4.ip_local_port_range=1024 65535
    deploy:
      resources:
        limits:
          cpus: "1.0"

networks:
  default:
    name: "sharkbench-benchmark-network"
    external: true
"#;

pub struct DockerFileManipulation {
    pub initial_from_version: String,
    pub new_from_version: String,
}

pub struct BenchmarkResult {
    pub time_median: i64,
    pub memory_median: i64,
    pub additional_data: IndexMap<String, AdditionalData>,
}

#[derive(Clone)]
pub enum AdditionalData {
    Int(i32),
    Float(f64),
}

impl Debug for AdditionalData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_additional_data(self, f)
    }
}

impl Display for AdditionalData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_additional_data(self, f)
    }
}

fn format_additional_data(data: &AdditionalData, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", match data {
        AdditionalData::Int(value) => value.to_string(),
        AdditionalData::Float(value) => value.to_string(),
    })
}

pub fn run_benchmark<F>(
    dir: &str,
    stats_reader: &mut crate::utils::docker_stats::DockerStatsReader,
    docker_file_manipulation: &Option<DockerFileManipulation>,
    rounds: usize,
    on_iteration: F,
) -> BenchmarkResult
    where F: Fn() -> Result<IndexMap<String, AdditionalData>, Box<dyn std::error::Error>>
{
    let original_docker_file = match docker_file_manipulation {
        Some(manipulation) => {
            let docker_file_content: String = fs::read_to_string(format!("{}/Dockerfile", dir)).expect("Failed to read Dockerfile");
            let new_docker_file_content = update_docker_file_with_version(&docker_file_content, &manipulation.initial_from_version, &manipulation.new_from_version);
            fs::write(format!("{}/Dockerfile", dir), new_docker_file_content).expect("Failed to write new Dockerfile");
            Some(docker_file_content)
        }
        None => None,
    };

    let mut execution_times: Vec<i64> = Vec::new();
    let mut memory_usages: Vec<i64> = Vec::new();
    let mut additional_data: Vec<IndexMap<String, AdditionalData>> = Vec::new();

    run_docker_compose(dir, Some(COMPOSE_FILE), || {
        println!(" -> Running benchmark");
        let mut fail_count = 0;
        let mut first_run = true; // For warm-up
        while execution_times.len() < rounds {
            let start = std::time::Instant::now();
            stats_reader.start();

            let result = match on_iteration() {
                Ok(result) => result,
                Err(e) => {
                    println!(" -> Error: {}", e);
                    fail_count += 1;
                    if fail_count > 10 {
                        panic!("Too many errors");
                    }
                    thread::sleep(Duration::from_secs(1));
                    println!("Retrying...");
                    continue;
                }
            };

            stats_reader.stop();

            let elapsed = start.elapsed().as_millis() as i64;
            let memory_usage = stats_reader.median_memory();

            if first_run {
                first_run = false;
                println!(" -> [Warmup]:    t = {} ms, RAM = {}, {:?}", elapsed, memory_usage.bytes_to_string(), result);
                continue;
            }

            println!(" -> [Result #{}]: t = {} ms, RAM = {}, {:?}", execution_times.len() + 1, elapsed, memory_usage.bytes_to_string(), result);
            execution_times.push(elapsed);
            memory_usages.push(memory_usage);
            additional_data.push(result);
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Reset Dockerfile
    if let Some(original_docker_file) = original_docker_file {
        fs::write(format!("{}/Dockerfile", dir), original_docker_file).expect("Failed to reset Dockerfile");
    }

    // Calculate medians
    execution_times.sort();
    memory_usages.sort();
    let time_median = execution_times[execution_times.len() / 2];
    let memory_median = memory_usages[memory_usages.len() / 2];
    let additional_data_median = {
        // find total unique keys
        let mut keys: Vec<String> = Vec::new();
        for data in &additional_data {
            for key in data.keys() {
                if !keys.contains(key) {
                    keys.push(key.clone());
                }
            }
        }

        // for each key, find the median value
        let mut map: IndexMap<String, AdditionalData> = IndexMap::new();

        for key in keys {
            let mut values: Vec<AdditionalData> = Vec::new();
            for data in &additional_data {
                if let Some(value) = data.get(&key) {
                    values.push(value.clone());
                }
            }
            values.sort_by(|a, b| {
                match (a, b) {
                    (AdditionalData::Int(a), AdditionalData::Int(b)) => a.cmp(b),
                    (AdditionalData::Float(a), AdditionalData::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
                    _ => panic!("Invalid type"),
                }
            });
            map.insert(key, values[values.len() / 2].clone());
        }

        map
    };

    return BenchmarkResult {
        time_median,
        memory_median,
        additional_data: additional_data_median,
    };
}

trait SizeFormat {
    fn bytes_to_string(&self) -> String;
}

impl SizeFormat for i64 {
    fn bytes_to_string(&self) -> String {
        let kb = *self as f64 / 1024.0;
        if kb < 1024.0 {
            return format!("{:.2} KB", kb);
        }
        let mb = kb / 1024.0;
        if mb < 1024.0 {
            return format!("{:.2} MB", mb);
        }
        let gb = mb / 1024.0;
        format!("{:.2} GB", gb)
    }
}

fn update_docker_file_with_version(docker_file_content: &str, current_version: &str, new_version: &str) -> String {
    docker_file_content
        .lines()
        .map(|line| {
            if line.starts_with("FROM ") {
                line.replacen(current_version, new_version, 1)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
