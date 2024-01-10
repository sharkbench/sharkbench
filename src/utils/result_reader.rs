use std::collections::{HashMap, HashSet};
use std::fs;

pub struct ResultMap {
    /// Map of language to a set of directories that have been benchmarked
    pub computation: HashMap<String, HashSet<String>>,

    /// Map of language to a set of directories that have been benchmarked
    pub web: HashMap<String, HashSet<String>>,
}

impl Default for ResultMap {
    fn default() -> Self {
        ResultMap {
            computation: HashMap::new(),
            web: HashMap::new(),
        }
    }
}

pub fn read_existing_result_map() -> ResultMap {
    let mut result_map = ResultMap {
        computation: HashMap::new(),
        web: HashMap::new(),
    };

    read_from_csv("result/computation_result.csv", 3, &mut result_map.computation);
    read_from_csv("result/web_result.csv", 7, &mut result_map.web);

    result_map
}

fn read_from_csv(csv_path: &str, dir_column: usize, map: &mut HashMap<String, HashSet<String>>) {
    let csv_content: String = fs::read_to_string(csv_path).unwrap_or(String::new());
    read_from_csv_content(csv_content.as_str(), dir_column, map);
}

fn read_from_csv_content(csv_content: &str, dir_column: usize, map: &mut HashMap<String, HashSet<String>>) {
    csv_content.split("\n").skip(1).for_each(|line| {
        let line = line.trim();
        if line.is_empty() {
            return;
        }

        let mut columns = line.split(",");
        let mut full_dir = columns.nth(dir_column).unwrap().split("/");
        let language = full_dir.nth(0).unwrap();
        let dir = full_dir.nth(0).unwrap();

        let language_map = map.entry(language.to_string()).or_insert(HashSet::new());
        language_map.insert(dir.to_string());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_from_csv_content() {
        let mut map = HashMap::new();
        read_from_csv_content(
r#"a,dir,b
1,java/springboot-java,1
2,php/symfony-php,2
3,rust/axum-rust,3
4,rust/actix-rust,4
"#,
            1,
            &mut map,
        );

        assert_eq!(map.len(), 3);
        assert_eq!(*map.get("java").unwrap(), HashSet::from(["springboot-java".to_string()]));
        assert_eq!(*map.get("php").unwrap(), HashSet::from(["symfony-php".to_string()]));
        assert_eq!(*map.get("rust").unwrap(), HashSet::from(["axum-rust".to_string(), "actix-rust".to_string()]));
    }
}
