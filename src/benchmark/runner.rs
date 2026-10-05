use crate::benchmark::computation::benchmark_computation;
use crate::benchmark::task::{BenchmarkDir, BenchmarkTask};
use crate::benchmark::web::{benchmark_web, prepare_requests};
use crate::utils::docker_runner::{run_docker_compose, StartMode};
use crate::utils::docker_stats::DockerStatsReader;
use std::fmt::Display;

const WEB_DATASOURCE_DIR: &str = "src/benchmark/web/data";

pub struct RunConfig {
    /// Reduce the benchmark time to a minimum and do not save results
    pub validate: bool,

    /// Print more information
    pub verbose: bool,
}

/// Runs the tasks in order.
/// The web data source is only started if there is at least one web task.
pub fn run_tasks(tasks: Vec<BenchmarkTask>, reader: &mut DockerStatsReader, config: &RunConfig) {
    let mut progress = Progress::new(tasks.len());

    let mut computation_tasks = Vec::new();
    let mut web_tasks = Vec::new();
    for task in tasks {
        match task {
            BenchmarkTask::Computation(task) => computation_tasks.push(task),
            BenchmarkTask::Web(task) => web_tasks.push(task),
        }
    }

    for task in &computation_tasks {
        progress.next(&task, &task.dir, || task.meta_data.print_info());
        benchmark_computation(task, reader, config);
    }

    if web_tasks.is_empty() {
        return;
    }

    run_docker_compose(
        WEB_DATASOURCE_DIR,
        StartMode::Detached { ready_port: None },
        None,
        false,
        |_| {
            let requests = prepare_requests();
            for task in &web_tasks {
                progress.next(&task, &task.dir, || task.meta_data.print_info());
                benchmark_web(task, &requests, reader, config);
            }
        },
    );
}

struct Progress {
    current: usize,
    total: usize,
    previous_dir: Option<String>,
}

impl Progress {
    fn new(total: usize) -> Self {
        Progress {
            current: 0,
            total,
            previous_dir: None,
        }
    }

    /// Prints the next task. The meta data is only printed once per directory.
    fn next(&mut self, task: &impl Display, dir: &BenchmarkDir, print_info: impl FnOnce()) {
        self.current += 1;
        println!();
        println!(
            " -> [{}/{}] Benchmarking {}",
            self.current, self.total, task
        );

        if self.previous_dir.as_deref() != Some(dir.path.as_str()) {
            self.previous_dir = Some(dir.path.clone());
            print_info();
        }
    }
}
