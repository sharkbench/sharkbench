use std::time::Duration;
use std::{fs, thread};

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
}

pub fn run_benchmark<F>(
    dir: &str,
    stats_reader: &mut crate::utils::docker_stats::DockerStatsReader,
    docker_file_manipulation: &Option<DockerFileManipulation>,
    rounds: usize,
    on_iteration: F,
) -> BenchmarkResult
    where F: Fn() -> Result<(), Box<dyn std::error::Error>>
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

    crate::utils::docker_runner::run_docker_compose(dir, Some(COMPOSE_FILE), || {
        println!(" -> Running benchmark");
        let mut fail_count = 0;
        let mut first_run = true; // For warm-up
        while execution_times.len() < rounds {
            let start = std::time::Instant::now();
            stats_reader.start();

            if let Err(e) = on_iteration() {
                println!(" -> Error: {}", e);
                fail_count += 1;
                if fail_count > 10 {
                    panic!("Too many errors");
                }
                thread::sleep(Duration::from_secs(1));
                println!("Retrying...");
                continue;
            }

            stats_reader.stop();

            let elapsed = start.elapsed().as_millis() as i64;
            let memory_usage = stats_reader.median_memory();

            if first_run {
                first_run = false;
                println!(" -> [Warmup]:    t = {} ms, RAM = {}", elapsed, memory_usage.bytes_to_string());
                continue;
            }

            println!(" -> [Result #{}]: t = {} ms, RAM = {}", execution_times.len() + 1, elapsed, memory_usage.bytes_to_string());
            execution_times.push(elapsed);
            memory_usages.push(memory_usage);
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

    return BenchmarkResult {
        time_median,
        memory_median,
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
