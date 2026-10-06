use crate::utils::meta_data_parser::{BenchmarkMetaData, WebBenchmarkMetaData};
use crate::utils::result_reader::{ExistingResult, ResultMap};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

/// A single benchmark run, i.e. one row in the result CSV.
pub enum BenchmarkTask {
    Computation(ComputationTask),
    Web(WebTask),
}

pub struct ComputationTask {
    pub dir: BenchmarkDir,
    pub meta_data: Rc<BenchmarkMetaData>,
    pub language_version: String,
}

pub struct WebTask {
    pub dir: BenchmarkDir,
    pub meta_data: Rc<WebBenchmarkMetaData>,
    pub language_version: String,
    pub framework_version: String,
}

/// The directory of a benchmark variant.
#[derive(Debug, Clone)]
pub struct BenchmarkDir {
    /// Language directory name, e.g. `rust`
    pub language: String,

    /// Variant directory name, e.g. `axum-0.8.9-rust-1.98.1`
    pub variant: String,

    /// Full path, e.g. `benchmark/web/rust/axum-0.8.9-rust-1.98.1`
    pub path: String,
}

impl BenchmarkDir {
    /// The path as stored in the result CSV, e.g. `rust/axum-0.8.9-rust-1.98.1`
    pub fn result_path(&self) -> String {
        format!("{}/{}", self.language, self.variant)
    }
}

impl BenchmarkTask {
    /// Returns true if the result CSV already contains a result for this task.
    pub fn has_result(&self, existing: &ResultMap) -> bool {
        match self {
            BenchmarkTask::Computation(task) => find_existing(&existing.computation, &task.dir)
                .is_some_and(|existing| {
                    existing.language_versions.contains(&task.language_version)
                }),
            BenchmarkTask::Web(task) => {
                find_existing(&existing.web, &task.dir).is_some_and(|existing| {
                    existing.language_versions.contains(&task.language_version)
                        && existing
                            .framework_versions
                            .contains(&task.framework_version)
                })
            }
        }
    }
}

fn find_existing<'a>(
    map: &'a HashMap<String, HashMap<String, ExistingResult>>,
    dir: &BenchmarkDir,
) -> Option<&'a ExistingResult> {
    map.get(&dir.language)?.get(&dir.variant)
}

impl Display for BenchmarkTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BenchmarkTask::Computation(task) => task.fmt(f),
            BenchmarkTask::Web(task) => task.fmt(f),
        }
    }
}

impl Display for ComputationTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({} v{})",
            self.dir.path, self.meta_data.mode, self.language_version
        )
    }
}

impl Display for WebTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({} v{} / {} v{})",
            self.dir.path,
            self.meta_data.mode,
            self.language_version,
            self.meta_data.framework,
            self.framework_version
        )
    }
}
