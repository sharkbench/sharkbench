use crate::benchmark::task::{BenchmarkDir, BenchmarkTask, ComputationTask, WebTask};
use crate::utils::copy_files::COMMON_DIR;
use crate::utils::meta_data_parser::{BenchmarkMetaData, WebBenchmarkMetaData};
use std::fs;
use std::rc::Rc;

const COMPUTATION_DIR: &str = "benchmark/computation";
const WEB_DIR: &str = "benchmark/web";

/// Which tasks to plan.
pub struct TaskFilter {
    /// Include computation benchmarks
    pub computation: bool,

    /// Include web benchmarks
    pub web: bool,

    pub selection: Selection,

    /// Only include the last (highest) version combination of each benchmark
    pub only_latest: bool,
}

/// Which benchmark directories to plan.
pub enum Selection {
    /// Every variant of every language
    All,

    /// Every variant of a language, e.g. `rust`
    Language(String),

    /// A variant (`<language>/<variant>`), see [resolve_only_dir]
    Only(String),
}

/// Returns every task matching the filter, computation tasks first.
/// Every version combination declared in `benchmark.yaml` becomes its own task,
/// or only the last one if [TaskFilter::only_latest] is set.
pub fn plan_tasks(filter: &TaskFilter) -> Vec<BenchmarkTask> {
    let mut tasks = Vec::new();

    if filter.computation {
        for dir in select_dirs(COMPUTATION_DIR, &filter.selection) {
            let meta_data = Rc::new(
                BenchmarkMetaData::read_from_directory(&dir.path)
                    .expect(&format!("Failed to read meta data: {}", dir.path)),
            );
            for language_version in versions(&meta_data.language_version, filter.only_latest) {
                tasks.push(BenchmarkTask::Computation(ComputationTask {
                    dir: dir.clone(),
                    meta_data: Rc::clone(&meta_data),
                    language_version: language_version.clone(),
                }));
            }
        }
    }

    if filter.web {
        for dir in select_dirs(WEB_DIR, &filter.selection) {
            let meta_data = Rc::new(
                WebBenchmarkMetaData::read_from_directory(&dir.path)
                    .expect(&format!("Failed to read meta data: {}", dir.path)),
            );
            for language_version in versions(&meta_data.language_version, filter.only_latest) {
                for framework_version in versions(&meta_data.framework_version, filter.only_latest)
                {
                    tasks.push(BenchmarkTask::Web(WebTask {
                        dir: dir.clone(),
                        meta_data: Rc::clone(&meta_data),
                        language_version: language_version.clone(),
                        framework_version: framework_version.clone(),
                    }));
                }
            }
        }
    }

    tasks
}

/// Returns the versions to plan.
/// Versions are declared in ascending order, so the latest one is the last.
fn versions(versions: &[String], only_latest: bool) -> &[String] {
    if only_latest && !versions.is_empty() {
        &versions[versions.len() - 1..]
    } else {
        versions
    }
}

/// Returns the variant directories inside `base_dir` matching the selection.
fn select_dirs(base_dir: &str, selection: &Selection) -> Vec<BenchmarkDir> {
    match selection {
        Selection::All => sub_dirs(base_dir)
            .into_iter()
            .flat_map(|language| variant_dirs(base_dir, &language))
            .collect(),
        Selection::Language(language) => variant_dirs(base_dir, language),
        Selection::Only(dir) => resolve_only_dir(base_dir, dir),
    }
}

/// Resolves the `--only` argument (`<language>/<variant>`) inside `base_dir`.
///
/// An exact directory match is preferred.
/// Otherwise, the variant is treated as an unversioned prefix,
/// e.g. `rust/rama` matches `rust/rama-0.4-rust-1.98.1`.
fn resolve_only_dir(base_dir: &str, dir: &str) -> Vec<BenchmarkDir> {
    let parts: Vec<&str> = dir.split('/').collect();
    if parts.len() != 2 {
        panic!("Invalid directory format. Expected <language>/<variant>");
    }
    let (language, query) = (parts[0], parts[1]);

    let available = variant_dirs(base_dir, language);
    let variants = match_variants(&available, query);
    if variants.is_empty() {
        let names: Vec<&String> = available.iter().map(|variant| &variant.variant).collect();
        panic!(
            "No benchmark found for {}. Available in {}: {:?}",
            dir, language, names
        );
    }

    variants
}

/// Returns the variants matching `query`:
/// the exact match if there is one, otherwise all variants starting with `<query>-`.
fn match_variants(available: &[BenchmarkDir], query: &str) -> Vec<BenchmarkDir> {
    if let Some(exact) = available.iter().find(|variant| variant.variant == query) {
        return vec![exact.clone()];
    }

    let prefix = format!("{}-", query);
    available
        .iter()
        .filter(|variant| variant.variant.starts_with(&prefix))
        .cloned()
        .collect()
}

/// Returns each benchmark variant directory of a language,
/// excluding the common directory.
fn variant_dirs(base_dir: &str, language: &str) -> Vec<BenchmarkDir> {
    let language_dir = format!("{}/{}", base_dir, language);
    sub_dirs(&language_dir)
        .into_iter()
        .filter(|variant| variant != COMMON_DIR)
        .map(|variant| BenchmarkDir {
            language: language.to_string(),
            // Always use forward slashes so the path stored in the result is platform independent
            path: format!("{}/{}", language_dir, variant),
            variant,
        })
        .collect()
}

/// Returns the name of each subdirectory in `dir`, sorted by name.
fn sub_dirs(dir: &str) -> Vec<String> {
    let mut dirs: Vec<String> = fs::read_dir(dir)
        .expect(&format!("Could not read directory {}", dir))
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.file_name().to_str().unwrap().to_owned())
        .collect();
    // fs::read_dir does not guarantee any order
    dirs.sort();
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variants(names: &[&str]) -> Vec<BenchmarkDir> {
        names
            .iter()
            .map(|name| BenchmarkDir {
                language: "rust".to_string(),
                variant: name.to_string(),
                path: format!("benchmark/web/rust/{}", name),
            })
            .collect()
    }

    fn names(variants: Vec<BenchmarkDir>) -> Vec<String> {
        variants
            .into_iter()
            .map(|variant| variant.variant)
            .collect()
    }

    #[test]
    fn should_return_all_versions() {
        let all = vec!["1".to_string(), "2".to_string()];
        assert_eq!(versions(&all, false), &all[..]);
    }

    #[test]
    fn should_return_latest_version() {
        let all = vec!["1".to_string(), "2".to_string()];
        assert_eq!(versions(&all, true), &["2".to_string()]);
    }

    #[test]
    fn should_prefer_exact_match() {
        let available = variants(&["ktor-3", "ktor-3-cio-native"]);
        assert_eq!(names(match_variants(&available, "ktor-3")), vec!["ktor-3"]);
    }

    #[test]
    fn should_match_unversioned() {
        let available = variants(&["rama-0.4-rust-1.98.1", "axum-0.8.9-rust-1.98.1"]);
        assert_eq!(
            names(match_variants(&available, "rama")),
            vec!["rama-0.4-rust-1.98.1"]
        );
    }

    #[test]
    fn should_match_multiple() {
        let available = variants(&["vertx-4-semeru-11", "vertx-4-semeru-21", "vertxx-1"]);
        assert_eq!(
            names(match_variants(&available, "vertx")),
            vec!["vertx-4-semeru-11", "vertx-4-semeru-21"]
        );
    }

    #[test]
    fn should_not_match_partial_name() {
        let available = variants(&["express-5-bun-1"]);
        assert!(match_variants(&available, "bun").is_empty());
        assert!(match_variants(&available, "expr").is_empty());
    }
}
