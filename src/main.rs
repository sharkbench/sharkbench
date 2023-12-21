use std::time::Duration;
use crate::utils::docker_stats;

mod benchmark {
    pub mod benchmark;
}

mod utils {
    pub mod docker_runner;
    pub mod docker_stats;
}

const CONTAINER_NAME: &str = "benchmark";

fn main() {
    let mut reader = docker_stats::DockerStatsReader::new(CONTAINER_NAME);
    reader.start();
    let _ = reader.run();

    benchmark::benchmark::run_benchmark();

    // Do other work...
    std::thread::sleep(Duration::from_secs(5));

    println!("Median memory usage: {}", reader.median_memory());

    reader.stop();
    reader.dispose();
}
