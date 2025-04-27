use std::fs;
use indexmap::IndexMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BenchmarkMetaData {
    pub language: String,

    pub mode: String,

    #[serde(rename = "version")]
    pub language_version: Vec<String>,

    #[serde(rename = "version_regex")]
    pub language_version_regex: Option<IndexMap<String, String>>,

    #[serde(default = "default_as_false")]
    pub extended_warmup: bool,

    pub runs: Option<usize>,

    pub copy: Option<Vec<CopyValue>>,
}

#[derive(Serialize, Deserialize)]
pub struct WebBenchmarkMetaData {
    pub language: String,

    pub mode: String,

    #[serde(rename = "version")]
    pub language_version: Vec<String>,

    #[serde(rename = "version_regex")]
    pub language_version_regex: Option<IndexMap<String, String>>,

    pub framework: String,

    #[serde(default = "default_as_false")]
    pub framework_stdlib: bool,

    pub framework_website: String,

    pub framework_flavor: String,

    pub framework_version: Vec<String>,

    pub framework_version_regex: Option<IndexMap<String, String>>,

    #[serde(default = "default_as_false")]
    pub extended_warmup: bool,

    pub concurrency: Option<usize>,

    pub copy: Option<Vec<CopyValue>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum CopyValue {
    /// Copy a file without changing its path.
    /// Example: 'main.dart'
    Primitive(String),
    
    /// Copy a file and change its path.
    /// Example: 'Main.java': 'src/main/java/example/Main.java'
    Map(IndexMap<String, String>),
}

fn default_as_false() -> bool {
    false
}

impl BenchmarkMetaData {
    pub fn print_info(&self) {
        println!(" - Language: {}", self.language);
        println!(" - Mode: {}", self.mode);
        println!(" - Language version: {:?}", self.language_version);
        println!(" - Language version regex: {}", self.language_version_regex.debug_serialize());
        println!();
    }

    pub fn read_from_directory(dir: &str) -> Result<BenchmarkMetaData, serde_yaml::Error> {
        let contents = fs::read_to_string(format!("{}/benchmark.yaml", dir)).expect("Failed to read benchmark.yaml");
        serde_yaml::from_str(&contents)
    }
}

impl WebBenchmarkMetaData {
    pub fn print_info(&self) {
        println!(" - Language: {}", self.language);
        println!(" - Mode: {}", self.mode);
        println!(" - Language version: {:?}", self.language_version);
        println!(" - Language version regex: {}", self.language_version_regex.debug_serialize());
        println!(" - Framework: {}", self.framework);
        println!(" - Framework stdlib: {}", self.framework_stdlib);
        println!(" - Framework website: {}", self.framework_website);
        println!(" - Framework flavor: {}", self.framework_flavor);
        println!(" - Framework version: {:?}", self.framework_version);
        println!(" - Framework version regex: {:?}", self.framework_version_regex.debug_serialize());
        println!(" - Concurrency: {:?}", self.concurrency);
        println!(" - Copy: {:?}", self.copy);
        println!();
    }

    pub fn read_from_directory(dir: &str) -> Result<WebBenchmarkMetaData, serde_yaml::Error> {
        let contents = fs::read_to_string(format!("{}/benchmark.yaml", dir)).expect("Failed to read benchmark.yaml");
        serde_yaml::from_str(&contents)
    }
}

trait VersionRegexSerializer {
    fn debug_serialize(&self) -> String;
}

impl VersionRegexSerializer for Option<IndexMap<String, String>> {
    fn debug_serialize(&self) -> String {
        match self {
            Some(regex) => format!("{:?}", regex),
            None => "Default".to_string(),
        }
    }
}
