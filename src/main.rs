use std::time::Duration;
use crate::utils::docker_stats;

mod utils {
    pub mod docker_stats;
    pub mod shell;
}

fn main() {
    let mut reader = docker_stats::DockerStatsReader::new("web_data_source".to_string());
    reader.start();
    let _ = reader.run();


    // Do other work...
    std::thread::sleep(Duration::from_secs(5));

    println!("Median memory usage: {}", reader.median_memory());

    reader.stop();
    reader.dispose();
}
