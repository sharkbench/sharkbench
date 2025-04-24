use indexmap::IndexMap;
use regex::RegexBuilder;

const DEFAULT_REGEX_KEYWORD: &str = "DEFAULT_DOCKER_REGEX";
const DEFAULT_REGEX_STRING: &str = r"^FROM.*:([\d.]+)(?:-.*)?(?: AS \w+)?$";

pub struct VersionMigrator {
    transformations: Vec<Transformation>,
    initial_version: String,
    target_version: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Transformation {
    path: String,
    original: Option<String>,
    regex: String,
}

impl VersionMigrator {
    pub fn new(
        dir: &str,
        regex: Option<IndexMap<String, String>>,
        initial_version: String,
        target_version: String,
    ) -> VersionMigrator {
        VersionMigrator {
            transformations: build_initial_transformation(dir, regex),
            initial_version,
            target_version,
        }
    }

    pub fn migrate(&mut self) {
        self.load_original_contents();

        for t in &mut self.transformations {
            let contents = t.original.as_ref().unwrap();

            match migrate_contents(contents, &t.regex, &self.initial_version, &self.target_version) {
                Ok(new_contents) => {
                    std::fs::write(&t.path, new_contents).expect(format!("Could not write {}", t.path).as_str());
                }
                Err(e) => {
                    match e {
                        MigrationError::VersionNotFound => {
                            panic!("Expected {} in {} but found none.", self.initial_version, t.path);
                        }
                        MigrationError::InvalidVersion(version) => {
                            panic!("Expected {} in {} but found {}.", self.initial_version, t.path, version);
                        }
                        MigrationError::InvalidRegex(regex) => {
                            panic!("Regex in file {} is invalid: {}", t.path, regex);
                        }
                    }
                }
            }
        }
    }

    pub fn restore(&self) {
        for t in &self.transformations {
            let contents = t.original.as_ref().expect(format!("Could not restore {} (original not found). This should not happen.", t.path).as_str());
            std::fs::write(&t.path, contents).expect(format!("Could not write {}", t.path).as_str());
        }
    }

    /// Store the contents in self.transformations.original
    fn load_original_contents(&mut self) {
        for t in &mut self.transformations {
            let contents = std::fs::read_to_string(&t.path).expect(format!("Could not read {}", t.path).as_str());
            t.original = Some(contents);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum MigrationError {
    InvalidRegex(String),
    VersionNotFound,
    InvalidVersion(String),
}

fn build_initial_transformation(dir: &str, regex: Option<IndexMap<String, String>>) -> Vec<Transformation> {
    regex
        .unwrap_or_else(|| {
            let mut map = IndexMap::new();
            map.insert("Dockerfile".to_string(), DEFAULT_REGEX_STRING.to_string());
            map
        })
        .iter()
        .map(|(path, regex)| Transformation {
            path: format!("{}/{}", dir, path),
            original: None,
            regex: match regex.as_str() {
                DEFAULT_REGEX_KEYWORD => DEFAULT_REGEX_STRING.to_string(),
                _ => regex.to_string(),
            },
        })
        .collect()
}

/// Migrate the contents of a file.
/// Expects the regex to capture exactly one group containing the (old) version.
/// It will panic if the group does not exist or if it contains a different initial version.
fn migrate_contents(original: &str, regex: &str, initial_version: &str, target_version: &str) -> Result<String, MigrationError> {
    let regex = RegexBuilder::new(regex)
        .multi_line(true)
        .build()
        .map_err(|e| MigrationError::InvalidRegex(format!("Could not compile regex {}: {}", regex, e)))?;

    let mut replaced = false;
    let mut detected_version: Option<String> = None;
    let new_contents = regex.replace_all(original, |caps: &regex::Captures| {
        let full_match = caps.get(0).unwrap();
        let version = caps.get(1).unwrap();
        if version.as_str() != initial_version {
            // return original string if the version is not the initial version
            detected_version = Some(version.as_str().to_string());
            return caps.get(0).unwrap().as_str().to_string();
        }

        let match_start = full_match.start();
        let start_index = version.start();
        replaced = true;
        format!("{}{}{}", &full_match.as_str()[..start_index - match_start], target_version, &full_match.as_str()[start_index - match_start + version.len()..])
    }).parse().or(Err(MigrationError::VersionNotFound))?;

    if !replaced {
        return if let Some(detected_version) = detected_version {
            Err(MigrationError::InvalidVersion(detected_version))
        } else {
            Err(MigrationError::VersionNotFound)
        };
    }

    Ok(new_contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_initial_transformation() {
        let dir = "test";
        let regex = None;
        let actual = build_initial_transformation(dir, regex);
        assert_eq!(vec![
            Transformation {
                path: "test/Dockerfile".to_string(),
                original: None,
                regex: DEFAULT_REGEX_STRING.to_string(),
            },
        ], actual);

        let dir = "test";
        let mut regex = IndexMap::new();
        regex.insert("Dockerfile2".to_string(), DEFAULT_REGEX_KEYWORD.to_string());
        let actual = build_initial_transformation(dir, Some(regex));
        assert_eq!(vec![
            Transformation {
                path: "test/Dockerfile2".to_string(),
                original: None,
                regex: DEFAULT_REGEX_STRING.to_string(),
            },
        ], actual);

        let dir = "test";
        let mut regex = IndexMap::new();
        regex.insert("Dockerfile3".to_string(), "my regex".to_string());
        regex.insert("Dockerfile4".to_string(), DEFAULT_REGEX_KEYWORD.to_string());
        let actual = build_initial_transformation(dir, Some(regex));
        assert_eq!(vec![
            Transformation {
                path: "test/Dockerfile3".to_string(),
                original: None,
                regex: "my regex".to_string(),
            },
            Transformation {
                path: "test/Dockerfile4".to_string(),
                original: None,
                regex: DEFAULT_REGEX_STRING.to_string(),
            },
        ], actual);
    }

    #[test]
    fn test_migrate_contents() {
        let initial_version = "1.0.0";
        let target_version = "2.0.0";
        let regex = DEFAULT_REGEX_STRING;

        let original = "FROM rust:1.0.0";
        let expected = "FROM rust:2.0.0";
        let actual = migrate_contents(original, regex, initial_version, target_version).unwrap();
        assert_eq!(expected, actual);

        let original = "FROM rust:1.0.0-rc1";
        let expected = "FROM rust:2.0.0-rc1";
        let actual = migrate_contents(original, regex, initial_version, target_version).unwrap();
        assert_eq!(expected, actual);

        let original = "FROM rust:1.0.0-rc1 AS builder";
        let expected = "FROM rust:2.0.0-rc1 AS builder";
        let actual = migrate_contents(original, regex, initial_version, target_version).unwrap();
        assert_eq!(expected, actual);

        let original = "FROM mcr.microsoft.com/dotnet/sdk:1.0.0 AS build";
        let expected = "FROM mcr.microsoft.com/dotnet/sdk:2.0.0 AS build";
        let actual = migrate_contents(original, regex, initial_version, target_version).unwrap();
        assert_eq!(expected, actual);

        let original = "FROM rust:1.0.0-rc1 AS builder\nFROM rust:1.0.0";
        let expected = "FROM rust:2.0.0-rc1 AS builder\nFROM rust:2.0.0";
        let actual = migrate_contents(original, regex, initial_version, target_version).unwrap();
        assert_eq!(expected, actual);

        let original = "FROM rust:";
        let actual = migrate_contents(original, regex, initial_version, target_version).unwrap_err();
        assert_eq!(MigrationError::VersionNotFound, actual);

        let original = "FROM rust:5.0.0";
        let actual = migrate_contents(original, regex, initial_version, target_version).unwrap_err();
        assert_eq!(MigrationError::InvalidVersion("5.0.0".to_string()), actual);
    }
}
