use crate::benchmark::planner::{Selection, TaskFilter};
use crate::benchmark::task::BenchmarkTask;
use crate::utils::result_reader::{
    CsvStructure, ResultFile, COMPUTATION_RESULT_FILE, WEB_RESULT_FILE,
};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs;

/// Identifies a result row.
#[derive(Debug, PartialEq, Eq, Hash)]
struct RowKey {
    /// See [crate::benchmark::task::BenchmarkDir::result_path]
    path: String,

    language_version: String,

    /// Only for web
    framework_version: Option<String>,
}

impl Display for RowKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.framework_version {
            Some(framework_version) => write!(
                f,
                "{} (v{} / v{})",
                self.path, self.language_version, framework_version
            ),
            None => write!(f, "{} (v{})", self.path, self.language_version),
        }
    }
}

/// Removes every row from the result CSVs that does not belong to any of the `tasks`.
///
/// Only rows within the scope of the filter are considered,
/// so `--lang rust` never removes rows of other languages.
///
/// Returns the number of removed rows. If `dry_run` is set, the files are not modified.
pub fn prune_results(tasks: &[BenchmarkTask], filter: &TaskFilter, dry_run: bool) -> usize {
    let mut planned_dirs: HashSet<String> = HashSet::new();
    let mut planned_rows: HashSet<RowKey> = HashSet::new();
    for task in tasks {
        let (dir, framework_version, language_version) = match task {
            BenchmarkTask::Computation(task) => (&task.dir, None, &task.language_version),
            BenchmarkTask::Web(task) => (
                &task.dir,
                Some(task.framework_version.clone()),
                &task.language_version,
            ),
        };
        planned_dirs.insert(dir.result_path());
        planned_rows.insert(RowKey {
            path: dir.result_path(),
            language_version: language_version.clone(),
            framework_version,
        });
    }

    let in_scope = |path: &str| match &filter.selection {
        Selection::All => true,
        Selection::Language(language) => path.split('/').next() == Some(language.as_str()),
        // A deleted directory cannot be resolved, so only the resolved directories are in scope.
        Selection::Only(_) => planned_dirs.contains(path),
    };
    let keep = |key: &RowKey| !in_scope(&key.path) || planned_rows.contains(key);

    let mut removed = 0;
    if filter.computation {
        removed += prune_file(&COMPUTATION_RESULT_FILE, &keep, dry_run);
    }
    if filter.web {
        removed += prune_file(&WEB_RESULT_FILE, &keep, dry_run);
    }
    removed
}

fn prune_file(file: &ResultFile, keep: &impl Fn(&RowKey) -> bool, dry_run: bool) -> usize {
    let Ok(content) = fs::read_to_string(file.path) else {
        return 0;
    };

    let (pruned, removed) = prune_content(&content, &file.structure, keep);
    if removed.is_empty() {
        return 0;
    }

    let verb = if dry_run { "Would remove" } else { "Removing" };
    println!(" -> {verb} {} rows from {}:", removed.len(), file.path);
    for key in &removed {
        println!("    - {key}");
    }

    if !dry_run {
        fs::write(file.path, pruned).expect(&format!("Failed to write {}", file.path));
    }

    removed.len()
}

/// Returns the content without the rows not passing `keep`, and the keys of the removed rows.
/// The header and malformed rows are always kept.
fn prune_content(
    content: &str,
    structure: &CsvStructure,
    keep: impl Fn(&RowKey) -> bool,
) -> (String, Vec<RowKey>) {
    let mut pruned = String::new();
    let mut removed = Vec::new();

    for (index, line) in content.lines().enumerate() {
        if index > 0 {
            if let Some(key) = row_key(line, structure) {
                if !keep(&key) {
                    removed.push(key);
                    continue;
                }
            }
        }
        pruned.push_str(line);
        pruned.push('\n');
    }

    (pruned, removed)
}

fn row_key(line: &str, structure: &CsvStructure) -> Option<RowKey> {
    let columns: Vec<&str> = line.trim().split(',').collect();
    let column = |index: usize| columns.get(index).map(|value| value.to_string());

    let framework_version = match structure.framework_version {
        Some(index) => Some(column(index)?),
        None => None,
    };
    Some(RowKey {
        path: column(structure.dir)?,
        language_version: column(structure.language_version)?,
        framework_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEB: CsvStructure = CsvStructure {
        dir: 3,
        language_version: 1,
        framework_version: Some(2),
    };

    const CONTENT: &str = r"lang,version,framework_version,path
Rust,1.80,0.7,rust/axum
Rust,1.80,0.8,rust/axum
Rust,1.80,4,rust/actix
Java,21,3,java/spring
";

    fn key(path: &str, language_version: &str, framework_version: &str) -> RowKey {
        RowKey {
            path: path.to_string(),
            language_version: language_version.to_string(),
            framework_version: Some(framework_version.to_string()),
        }
    }

    #[test]
    fn should_keep_everything() {
        let (pruned, removed) = prune_content(CONTENT, &WEB, |_| true);
        assert_eq!(pruned, CONTENT);
        assert!(removed.is_empty());
    }

    #[test]
    fn should_remove_rows() {
        let (pruned, removed) = prune_content(CONTENT, &WEB, |row| {
            row != &key("rust/axum", "1.80", "0.7") && row.path != "rust/actix"
        });
        assert_eq!(
            pruned,
            "lang,version,framework_version,path\nRust,1.80,0.8,rust/axum\nJava,21,3,java/spring\n"
        );
        assert_eq!(
            removed,
            vec![
                key("rust/axum", "1.80", "0.7"),
                key("rust/actix", "1.80", "4")
            ]
        );
    }

    #[test]
    fn should_keep_header_and_malformed_rows() {
        let content = "lang,version,framework_version,path\nRust,1.80\n";
        let (pruned, removed) = prune_content(content, &WEB, |_| false);
        assert_eq!(pruned, content);
        assert!(removed.is_empty());
    }

    #[test]
    fn should_read_computation_row_key() {
        let structure = CsvStructure {
            dir: 3,
            language_version: 2,
            framework_version: None,
        };
        assert_eq!(
            row_key("C,GCC,7,c/gcc-6,1545,401408", &structure),
            Some(RowKey {
                path: "c/gcc-6".to_string(),
                language_version: "7".to_string(),
                framework_version: None,
            })
        );
    }
}
