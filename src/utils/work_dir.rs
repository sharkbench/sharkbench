use crate::utils::docker_runner::IGNORED_DIRS;
use std::fs;
use std::path::Path;

/// Directory (relative to the repository root) in which the benchmarks are built and run.
/// The benchmark folder itself is never modified.
pub(crate) const WORK_DIR: &str = "tmp";

/// Replaces [WORK_DIR] with a copy of `src_dir` and returns its path.
pub(crate) fn prepare_work_dir(src_dir: &str) -> &'static str {
    remove_work_dir();
    copy_dir(Path::new(src_dir), Path::new(WORK_DIR));
    println!(" -> Copied {src_dir} to {WORK_DIR}");
    WORK_DIR
}

/// Deletes [WORK_DIR] if it exists.
pub(crate) fn remove_work_dir() {
    if Path::new(WORK_DIR).exists() {
        fs::remove_dir_all(WORK_DIR).expect(&format!("Failed to remove {WORK_DIR}"));
    }
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect(&format!("Failed to create directory {dst:?}"));
    for entry in fs::read_dir(src).expect(&format!("Failed to read directory {src:?}")) {
        let entry = entry.expect("Failed to read directory entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            // Excluded from the docker context anyway
            if IGNORED_DIRS.contains(&entry.file_name().to_string_lossy().as_ref()) {
                continue;
            }
            copy_dir(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).expect(&format!("Failed to copy {src_path:?}"));
        }
    }
}
