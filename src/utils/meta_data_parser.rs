use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BenchmarkMetaData {
    pub language: String,

    pub mode: String,

    #[serde(rename = "version")]
    pub language_version: Vec<String>,

    pub framework: Option<String>,

    pub framework_version: Option<Vec<String>>,
}

impl BenchmarkMetaData {
    pub fn print_info(&self) {
        println!(" - Language: {}", self.language);
        println!(" - Mode: {}", self.mode);
        println!(" - Language version: {:?}", self.language_version);
        if let Some(framework) = &self.framework {
            println!(" - Framework: {}", framework);
        }
        if let Some(framework_version) = &self.framework_version {
            println!(" - Framework version: {:?}", framework_version);
        }
        println!();
    }

    pub fn read_from_directory(dir: &str) -> Result<BenchmarkMetaData, serde_yaml::Error> {
        let contents = fs::read_to_string(format!("{}/_benchmark.yaml", dir)).expect("Failed to read _benchmark.yaml");
        serde_yaml::from_str(&contents)
    }
}
