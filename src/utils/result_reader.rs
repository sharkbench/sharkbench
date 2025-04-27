use std::collections::{HashMap, HashSet};
use std::fs;

pub struct ResultMap {
    /// Map of language to a set of directories that have been benchmarked
    /// Language -> Directory (Variant) -> ExistingResult
    pub computation: HashMap<String, HashMap<String, ExistingResult>>,

    /// Map of language to a set of directories that have been benchmarked
    /// Language -> Directory (Variant) -> ExistingResult
    pub web: HashMap<String, HashMap<String, ExistingResult>>,
}

pub struct ExistingResult {
    #[allow(dead_code)]
    pub language: String,

    /// A framework or platform.
    /// Example: express-4-nodejs-12, temurin-8
    #[allow(dead_code)]
    pub variant: String,

    /// List of available versions for the variant.
    pub language_versions: HashSet<String>,

    /// List of available versions for the framework (only for web).
    pub framework_versions: HashSet<String>,
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

    read_from_csv(
        "result/computation_result.csv",
        CsvStructure {
            dir: 3,
            language_version: 2,
            framework_version: None,
        },
        &mut result_map.computation,
    );

    read_from_csv(
        "result/web_result.csv",
        CsvStructure {
            dir: 9,
            language_version: 2,
            framework_version: Some(7),
        },
        &mut result_map.web,
    );

    result_map
}

struct CsvStructure {
    dir: usize,
    language_version: usize,
    framework_version: Option<usize>,
}

fn read_from_csv(
    csv_path: &str,
    csv_structure: CsvStructure,
    map: &mut HashMap<String, HashMap<String, ExistingResult>>,
) {
    let csv_content: String = fs::read_to_string(csv_path).unwrap_or(String::new());
    read_from_csv_content(csv_content.as_str(), csv_structure, map);
}

fn read_from_csv_content(
    csv_content: &str,
    csv_structure: CsvStructure,
    map: &mut HashMap<String, HashMap<String, ExistingResult>>,
) {
    let mut language_versions: HashMap<(String, String), HashSet<String>> = HashMap::new();
    let mut framework_versions: HashMap<(String, String), HashSet<String>> = HashMap::new();

    csv_content.split("\n").skip(1).for_each(|line| {
        let line = line.trim();
        if line.is_empty() {
            return;
        }

        let columns: Vec<&str> = line.split(",").collect();
        if columns.len() <= csv_structure.dir {
            return;
        }

        let full_dir: Vec<&str> = columns[csv_structure.dir].split("/").collect();
        if full_dir.len() != 2 {
            panic!(
                "Invalid directory format: {} (expected: lang/version)",
                full_dir.join("/")
            );
        }

        let language = full_dir[0];
        let variant = full_dir[1];

        // Language version
        {
            let language_version = columns[csv_structure.language_version].to_string();
            let key = (language.to_string(), variant.to_string());
            language_versions
                .entry(key.clone())
                .or_insert_with(HashSet::new)
                .insert(language_version);
        }

        // Framework version
        if let Some(framework_version_index) = csv_structure.framework_version {
            let framework_version = columns[framework_version_index].to_string();
            let key = (language.to_string(), variant.to_string());
            framework_versions
                .entry(key)
                .or_insert_with(HashSet::new)
                .insert(framework_version);
        }
    });

    // Final map construction
    for ((language, variant), lang_versions) in language_versions {
        let variant_map = map.entry(language.clone()).or_insert_with(HashMap::new);

        let existing_result = ExistingResult {
            language: language.clone(),
            variant: variant.clone(),
            language_versions: lang_versions.into_iter().collect(),
            framework_versions: framework_versions
                .get(&(language.clone(), variant.clone()))
                .map_or_else(HashSet::new, |set| set.iter().cloned().collect()),
        };

        variant_map.insert(variant, existing_result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_from_csv_content() {
        let mut map = HashMap::new();
        read_from_csv_content(
            r#"header1,header2,lang_version,header4,header5,header6,framework_version,header8,header9,dir
-,-,8,-,-,-,3,-,-,java/springboot-java
-,-,17,-,-,-,3,-,-,java/springboot-java
-,-,7,-,-,-,4.0,-,-,php/symfony-php
-,-,8,-,-,-,5.0,-,-,php/symfony-php
-,-,1.67,-,-,-,4,-,-,rust/actix-rust
"#,
            CsvStructure {
                dir: 9,
                language_version: 2,
                framework_version: Some(6),
            },
            &mut map,
        );

        // Check length of map
        assert_eq!(map.len(), 3);

        // Check Java
        let java_map = map.get("java").expect("Java should exist");
        assert_eq!(java_map.len(), 1);

        let springboot = java_map
            .get("springboot-java")
            .expect("springboot-java should exist");
        assert_eq!(springboot.language, "java");
        assert_eq!(springboot.variant, "springboot-java");
        assert_eq!(springboot.language_versions.len(), 2);
        assert!(springboot.language_versions.contains("8"));
        assert!(springboot.language_versions.contains("17"));
        assert_eq!(springboot.framework_versions.len(), 1);
        assert!(springboot.framework_versions.contains("3"));

        // Check PHP
        let php_map = map.get("php").expect("PHP should exist");
        assert_eq!(php_map.len(), 1);

        let symfony = php_map
            .get("symfony-php")
            .expect("symfony-php should exist");
        assert_eq!(symfony.language, "php");
        assert_eq!(symfony.variant, "symfony-php");
        assert_eq!(symfony.language_versions.len(), 2);
        assert!(symfony.language_versions.contains("7"));
        assert!(symfony.language_versions.contains("8"));
        assert_eq!(symfony.framework_versions.len(), 2);
        assert!(symfony.framework_versions.contains("4.0"));
        assert!(symfony.framework_versions.contains("5.0"));

        // Check Rust
        let rust_map = map.get("rust").expect("Rust should exist");
        assert_eq!(rust_map.len(), 1);

        let actix = rust_map.get("actix-rust").expect("actix-rust should exist");
        assert_eq!(actix.language, "rust");
        assert_eq!(actix.variant, "actix-rust");
        assert_eq!(actix.language_versions.len(), 1);
        assert!(actix.language_versions.contains("1.67"));
        assert_eq!(actix.framework_versions.len(), 1);
        assert!(actix.framework_versions.contains("4"));
    }
}
