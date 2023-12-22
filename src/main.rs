use crate::utils::docker_stats;

mod benchmark {
    pub mod computation {
        pub mod computation;
    }
    pub mod benchmark;
}

mod utils {
    pub mod docker_runner;
    pub mod docker_stats;
    pub mod meta_data_parser;
    pub mod result_writer;
}

const CONTAINER_NAME: &str = "benchmark";

fn main() {
    let mut reader = docker_stats::DockerStatsReader::new(CONTAINER_NAME);
    reader.run();

    benchmark::computation::computation::benchmark_computation(
        "benchmark/computation/dart/aot-2.14",
        &mut reader,
    );

    reader.stop();
    reader.dispose();
}
