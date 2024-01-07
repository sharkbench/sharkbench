#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    parts: Vec<u16>,
}

impl Version {
    pub fn parse(version: &str) -> Result<Version, String> {
        let parts: Vec<u16> = version.split(".").filter_map(|part| part.parse::<u16>().ok()).collect();

        if parts.is_empty() {
            return Err("Version must have at least one part".to_string());
        }

        Ok(Version {
            parts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_1_part() {
        assert_eq!(Version::parse("1"), Ok(Version {
            parts: vec![1],
        }));
    }

    #[test]
    fn should_parse_2_parts() {
        assert_eq!(Version::parse("1.2"), Ok(Version {
            parts: vec![1, 2],
        }));
    }

    #[test]
    fn should_parse_3_parts() {
        assert_eq!(Version::parse("1.2.3"), Ok(Version {
            parts: vec![1, 2, 3],
        }));
    }

    #[test]
    fn should_compare_1_part() {
        assert!(Version::parse("1").unwrap() < Version::parse("2").unwrap());
    }

    #[test]
    fn should_compare_2_parts() {
        assert!(Version::parse("1.2").unwrap() < Version::parse("1.10").unwrap());
        assert!(Version::parse("1.2").unwrap() > Version::parse("1.1").unwrap());
    }

    #[test]
    fn should_compare_3_parts() {
        assert!(Version::parse("1.2.3").unwrap() < Version::parse("1.2.20").unwrap());
        assert!(Version::parse("1.2.3").unwrap() > Version::parse("1.2.2").unwrap());
    }
}
