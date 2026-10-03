use crate::utils::docker_runner::{run_docker_compose, AttachedContainer, StartMode};
use crate::utils::percentile;
use crate::utils::version_migrator::VersionMigrator;
use indexmap::IndexMap;
use std::fmt::{Debug, Display};
use std::thread;
use std::time::Duration;

const CONTAINER_NAME: &str = "benchmark";

/// How the harness talks to the benchmark container.
pub enum Transport {
    /// The container runs an HTTP server. The compose file must publish its port
    /// or use host networking.
    Http,
    /// The container reads requests from stdin and writes responses to stdout.
    /// No port is needed.
    Stdio,
}

pub struct BenchmarkResult {
    /// Docker image build time in milliseconds (excluding base image downloads)
    pub build_time: i64,
    pub time_median: i64,
    pub memory_median: i64,
    pub memory_p99: i64,
    pub additional_data: IndexMap<String, AdditionalData>,
}

pub struct IterationResult {
    pub additional_data: IndexMap<String, AdditionalData>,
    pub debugging_data: IndexMap<String, AdditionalData>,
}

#[derive(Clone)]
pub enum AdditionalData {
    Int(i32),
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

fn format_additional_data(
    data: &AdditionalData,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(
        f,
        "{}",
        match data {
            AdditionalData::Int(value) => value.to_string(),
            // AdditionalData::Float(value) => value.to_string(),
        }
    )
}

pub fn run_benchmark<F>(
    dir: &str,
    compose_file: &str,
    stats_reader: &mut crate::utils::docker_stats::DockerStatsReader,
    mut version_migrations: Vec<&mut VersionMigrator>,
    warmup_rounds: usize,
    rounds: usize,
    transport: Transport,
    on_iteration: F,
) -> BenchmarkResult
where
    F: Fn(Option<&mut AttachedContainer>) -> Result<IterationResult, Box<dyn std::error::Error>>,
{
    for version_migrator in &mut version_migrations {
        version_migrator.migrate();
    }

    let mut execution_times: Vec<i64> = Vec::new();
    let mut memory_median: Vec<i64> = Vec::new();
    let mut memory_p99: Vec<i64> = Vec::new();
    let mut additional_data: Vec<IndexMap<String, AdditionalData>> = Vec::new();

    let start_mode = match transport {
        Transport::Http => StartMode::Detached {
            ready_delay: Duration::from_secs(5),
        },
        Transport::Stdio => StartMode::Attached {
            container_name: CONTAINER_NAME,
        },
    };
    let build_time = run_docker_compose(dir, start_mode, Some(compose_file), true, |container| {
        let mut container = container;
        println!(" -> Running benchmark");
        let mut fail_count = 0;
        let mut warmup_counter = 0;
        while execution_times.len() < rounds {
            if warmup_counter < warmup_rounds {
                println!(" -> [Warmup]: Running...");
            } else {
                println!(" -> [Run #{}]: Running...", execution_times.len() + 1);
            }

            let start = std::time::Instant::now();
            stats_reader.start();

            let result = match on_iteration(container.as_deref_mut()) {
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
            let memory_usage = stats_reader.get_memory_usage();

            if warmup_counter < warmup_rounds {
                warmup_counter += 1;
                println!(
                    " -> [Warmup]: t = {} ms, RAM = {}, {:?}, {:?}",
                    elapsed,
                    memory_usage.median.bytes_to_string(),
                    result.additional_data,
                    result.debugging_data,
                );
                continue;
            }

            println!(
                " -> [Run #{}]: t = {} ms, RAM = {}, {:?}, {:?}",
                execution_times.len() + 1,
                elapsed,
                memory_usage.median.bytes_to_string(),
                result.additional_data,
                result.debugging_data,
            );
            execution_times.push(elapsed);
            memory_median.push(memory_usage.median);
            memory_p99.push(memory_usage.p99);
            additional_data.push(result.additional_data);

            // Wait for 2 seconds to let the container cool down
            thread::sleep(Duration::from_secs(2));
        }
    });

    for version_migrator in &version_migrations {
        version_migrator.restore();
    }

    // Calculate medians
    execution_times.sort();
    let time_median = execution_times[execution_times.len() / 2];
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
                    // (AdditionalData::Float(a), AdditionalData::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
                    // _ => panic!("Invalid type"),
                }
            });
            map.insert(key, values[values.len() / 2].clone());
        }

        map
    };

    memory_median.sort();
    memory_p99.sort();

    BenchmarkResult {
        build_time: build_time.unwrap().as_millis() as i64,
        time_median,
        memory_median: percentile::p50(&memory_median),
        memory_p99: percentile::p99(&memory_p99),
        additional_data: additional_data_median,
    }
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
