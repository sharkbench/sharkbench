use crate::utils::meta_data_parser::CopyValue;
use std::fs;
use std::path::Path;

pub(crate) const COMMON_DIR: &str = "_common";

/// Copy files from the common directory next to `src_dir` to `work_dir`.
pub(crate) fn copy_files(src_dir: &str, work_dir: &str, files: &Vec<CopyValue>) {
    for file in files {
        let (src, dst) = match file {
            CopyValue::Primitive(src) => (src, src),
            CopyValue::Map(map) => map.get_index(0).expect("Failed to get index"),
        };

        let final_src = format!("{src_dir}/../{COMMON_DIR}/{src}");
        let final_dst = format!("{work_dir}/{dst}");
        let parent_dir = Path::new(&final_dst).parent().unwrap();
        fs::create_dir_all(parent_dir).expect("Failed to create directory");
        fs::copy(final_src, final_dst).expect("Failed to copy file");
        println!(" -> Copied {COMMON_DIR}/{src} to {dst}");
    }
}
