use std::process::{Command, Child};
use std::io::{BufReader, BufRead};
use std::sync::{Arc, Mutex};
use regex::Regex;
use crate::utils::percentile;

pub struct DockerStatsReader {
    is_tracking: Arc<Mutex<bool>>,
    process: Option<Child>,
    ram_usage: Arc<Mutex<Vec<i64>>>,
}

pub struct MemoryUsage {
    pub median: i64,
    pub p99: i64,
}

impl DockerStatsReader {
    pub fn new() -> DockerStatsReader {
        DockerStatsReader {
            is_tracking: Arc::new(Mutex::new(false)),
            process: None,
            ram_usage: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn run(&mut self, container_name: &'static str) {
        let is_tracking = Arc::clone(&self.is_tracking);
        let ram_usage = Arc::clone(&self.ram_usage);

        let child = Command::new("docker")
            .args(&["stats", "--format", "json"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("failed to execute process");

        self.process = Some(child);

        let stdout = self.process.as_mut().unwrap().stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        std::thread::spawn(move || {
            for line in reader.lines() {
                if *is_tracking.lock().unwrap() {
                    let line = line.unwrap();
                    let trimmed = match (line.find('{'), line.rfind('}')) {
                        (Some(start), Some(end)) => &line[start..=end],
                        _ => "",
                    };
                    if trimmed.is_empty() {
                        continue;
                    }

                    let json: serde_json::Value = match serde_json::from_str(trimmed) {
                        Ok(json) => json,
                        Err(e) => {
                            eprintln!("Failed to parse JSON: {} \n {}", trimmed, e);
                            continue;
                        }
                    };
                    let name = json["Name"].as_str().unwrap();
                    if name != container_name {
                        continue;
                    }
                    let mem_usage = get_bytes_of_ram(json["MemUsage"].as_str().unwrap());
                    ram_usage.lock().unwrap().push(mem_usage);
                }
            }

            println!("Docker stats reader thread finished");
        });
    }

    pub fn stop(&mut self) {
        let mut is_tracking = self.is_tracking.lock().unwrap();
        *is_tracking = false;
    }

    pub fn start(&mut self) {
        self.ram_usage.lock().unwrap().clear();
        let mut is_tracking = self.is_tracking.lock().unwrap();
        *is_tracking = true;
    }

    pub fn dispose(&mut self) {
        if let Some(child) = &mut self.process {
            child.kill().expect("failed to kill process");
        }
    }

    pub fn get_memory_usage(&self) -> MemoryUsage {
        let mut ram_usage = self.ram_usage.lock().unwrap();
        ram_usage.sort();
        if ram_usage.len() == 0 {
            return MemoryUsage {
                median: 0,
                p99: 0,
            };
        }
        MemoryUsage {
            median: percentile::p50(&ram_usage),
            p99: percentile::p99(&ram_usage),
        }
    }
}

/// Parses the given memory usage string and returns the number of bytes.
/// Example: "1.5GiB / 16GiB" -> 1610612736
fn get_bytes_of_ram(mem_usage: &str) -> i64 {
    let actual_usage = if mem_usage.contains('/') {
        // "used / limit"
        mem_usage.split('/').next().unwrap().trim()
    } else {
        // "used"
        mem_usage.trim()
    };

    let mem_usage_regex = Regex::new(r"(\d*\.?\d+)(\w+)").unwrap();
    let mem_usage_match = mem_usage_regex.captures(actual_usage).unwrap();
    let mem_usage_value = mem_usage_match.get(1).unwrap().as_str().parse::<f64>().unwrap();
    let mem_usage_unit = mem_usage_match.get(2).unwrap().as_str();

    match mem_usage_unit {
        "KiB" => (mem_usage_value * 1024.0) as i64,
        "MiB" => (mem_usage_value * 1024.0 * 1024.0) as i64,
        "GiB" => (mem_usage_value * 1024.0 * 1024.0 * 1024.0) as i64,
        _ => panic!("Unknown unit: {}", mem_usage_unit),
    }
}
