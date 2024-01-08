use std::process::{Command, Child};
use std::io::{BufReader, BufRead};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver};
use regex::Regex;
use crate::utils::percentile;

// The start of a stats line is marked by the following bytes.
// We ignore them because they are not valid JSON.
const STATS_PREFIX: &[u8] = &[27, 91, 50, 74, 27, 91, 72];

pub struct DockerStatsReader {
    container_name: &'static str,
    is_tracking: Arc<Mutex<bool>>,
    process: Option<Child>,
    ram_usage: Arc<Mutex<Vec<i64>>>,
}

pub struct MemoryUsage {
    pub median: i64,
    pub p99: i64,
}

impl DockerStatsReader {
    pub fn new(container_name: &'static str) -> DockerStatsReader {
        DockerStatsReader {
            container_name,
            is_tracking: Arc::new(Mutex::new(false)),
            process: None,
            ram_usage: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn run(&mut self) -> Receiver<()> {
        let (tx, rx) = mpsc::channel();
        let container_name = self.container_name;
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
                    let trimmed = if line.as_bytes().starts_with(STATS_PREFIX) {
                        // Trim off the prefix
                        &line[STATS_PREFIX.len()..]
                    } else {
                        &line[..]
                    };
                    if trimmed.is_empty() {
                        continue;
                    }

                    let json: serde_json::Value = match serde_json::from_str(trimmed) {
                        Ok(json) => json,
                        Err(e) => {
                            println!("Failed to parse JSON: {} \n {}", trimmed, e);
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
            tx.send(()).unwrap();
        });

        rx
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
/// Example: "1.5GiB" -> 1610612736
fn get_bytes_of_ram(mem_usage: &str) -> i64 {
    let mem_usage_regex = Regex::new(r"(\d*\.?\d+)(\w+)").unwrap();
    let mem_usage_match = mem_usage_regex.captures(mem_usage).unwrap();
    let mem_usage_value = mem_usage_match.get(1).unwrap().as_str().parse::<f64>().unwrap();
    let mem_usage_unit = mem_usage_match.get(2).unwrap().as_str();

    match mem_usage_unit {
        "KiB" => (mem_usage_value * 1024.0) as i64,
        "MiB" => (mem_usage_value * 1024.0 * 1024.0) as i64,
        "GiB" => (mem_usage_value * 1024.0 * 1024.0 * 1024.0) as i64,
        _ => panic!("Unknown unit: {}", mem_usage_unit),
    }
}
