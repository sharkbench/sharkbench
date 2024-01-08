use std::{fs, io};
use std::cmp::Ordering;
use std::path::Path;
use crate::utils::version::Version;

/// Writes the given result to the given `file_path`.
/// If the file does not exist, it will be created.
/// If the file exists, the result will be added or updated.
///
/// `descriptors` contains the keys and values for the first columns that describe the benchmark.
/// Example: `vec![("language", "Java"), ("version", "11")]`.
///
/// `values` contains the keys and values for the columns that contain the benchmark results.
/// Example: `vec![("time", "1234"), ("memory", "1234")]`.
pub fn write_result_to_file(
    file_path: &str,
    descriptors: &Vec<(&str, &str)>,
    values: &Vec<(&str, &str)>,
) -> io::Result<()> {
    println!(" -> Writing result:");
    for (key, value) in descriptors {
        println!("    - {}: {}", key, value);
    }
    for (key, value) in values {
        println!("    - {}: {}", key, value);
    }

    let descriptor_values: Vec<&str> = descriptors.iter().map(|(_, v)| *v).collect::<Vec<&str>>();
    let value_values: Vec<&str> = values.iter().map(|(_, v)| *v).collect::<Vec<&str>>();

    let original_contents: String = fs::read_to_string(file_path).unwrap_or(String::new());

    let contents: Vec<String> = if original_contents.is_empty() {
        vec!(format!("{},{}", descriptor_values.join(","), value_values.join(",")))
    } else {
        get_updated_contents(&original_contents, &descriptor_values, &value_values)
    };

    let header = {
        let descriptor_keys: String = descriptors.iter().map(|(k, _)| *k).collect::<Vec<&str>>().join(",");
        let values_keys: String = values.iter().map(|(k, _)| *k).collect::<Vec<&str>>().join(",");
        format!("{},{}", descriptor_keys, values_keys)
    };
    write_lines_to_file(file_path, &contents, &header).unwrap();

    Ok(())
}

fn get_updated_contents(original_contents: &str, descriptor_values: &Vec<&str>, value_values: &Vec<&str>) -> Vec<String> {
    let mut content_vectors: Vec<Vec<&str>> = original_contents
        .lines()
        .skip(1)
        .filter(|line| !line.starts_with(descriptor_values.join(",").as_str()))
        .map(|line| line.split(",").collect::<Vec<&str>>())
        .collect::<Vec<Vec<&str>>>();

    let new_line: Vec<&str> = {
        let mut new_line: Vec<&str> = Vec::new();
        new_line.extend_from_slice(descriptor_values);
        new_line.extend_from_slice(value_values);
        new_line
    };

    content_vectors.push(new_line);
    content_vectors.sort_by(compare_lines);

    return content_vectors
        .iter()
        .map(|line| line.join(","))
        .collect();
}

fn write_lines_to_file(file_path: &str, lines: &Vec<String>, header: &String) -> io::Result<()> {
    let mut output = String::new();

    // Header
    output.push_str(header);
    output.push('\n');

    // Data
    for line in lines {
        output.push_str(&line);
        output.push_str("\n");
    }

    // create directory if it does not exist
    let parent_dir = Path::new(file_path).parent().unwrap();
    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir).expect(format!("Failed to create directory {}", parent_dir.display()).as_str());
    }

    fs::write(file_path, output.as_bytes()).expect(format!("Failed to write {}", file_path).as_str());

    Ok(())
}

/// Compares two lines column-wise. Both lines must have the same number of columns.
/// If a column is detected as a version number, it will be compared as such.
/// Example: 1.2 < 1.10 (as opposed to 1.2 > 1.10 when comparing as strings)
fn compare_lines(a: &Vec<&str>, b: &Vec<&str>) -> Ordering {
    for i in 0..a.len() {
        let a_value = a[i];
        let b_value = b[i];

        match (Version::parse(a_value), Version::parse(b_value)) {
            (Ok(a_version), Ok(b_version)) => {
                match a_version.cmp(&b_version) {
                    Ordering::Equal => continue,
                    ordering => return ordering,
                }
            },
            _ => match a_value.cmp(&b_value) {
                Ordering::Equal => continue,
                ordering => return ordering,
            }
        }
    }

    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    mod compare_lines {
        use super::*;

        #[test]
        fn should_compare_single_entry_by_version() {
            assert_eq!(compare_lines(&vec!["1.2"], &vec!["1.10"]), Ordering::Less);
            assert_eq!(compare_lines(&vec!["1.2"], &vec!["1.1"]), Ordering::Greater);
        }

        #[test]
        fn should_compare_single_entry_alphabetically() {
            assert_eq!(compare_lines(&vec!["a"], &vec!["b"]), Ordering::Less);
            assert_eq!(compare_lines(&vec!["b"], &vec!["a"]), Ordering::Greater);
        }

        #[test]
        fn should_compare_second_entry_by_version() {
            assert_eq!(compare_lines(&vec!["a", "1.2"], &vec!["a", "1.10"]), Ordering::Less);
            assert_eq!(compare_lines(&vec!["a", "1.2"], &vec!["a", "1.1"]), Ordering::Greater);
        }

        #[test]
        fn should_compare_second_entry_alphabetically() {
            assert_eq!(compare_lines(&vec!["a", "a"], &vec!["a", "b"]), Ordering::Less);
            assert_eq!(compare_lines(&vec!["a", "b"], &vec!["a", "a"]), Ordering::Greater);
        }

        #[test]
        fn should_compare_first_entry_alphabetically() {
            assert_eq!(compare_lines(&vec!["a", "b"], &vec!["b", "a"]), Ordering::Less);
            assert_eq!(compare_lines(&vec!["b", "a"], &vec!["a", "b"]), Ordering::Greater);
        }
    }

    mod get_updated_contents {
        use super::*;

        #[test]
        fn should_update_single_line() {
            let original_contents = "a,b,c,d,e,f\n1,2,3,4,5,6\n";
            let descriptor_values = vec!["1", "2", "3"];
            let value_values = vec!["0", "0", "0"];

            let expected_contents = vec!["1,2,3,0,0,0"];
            assert_eq!(get_updated_contents(original_contents, &descriptor_values, &value_values), expected_contents);
        }

        #[test]
        fn should_add_new_line_if_not_exists() {
            let original_contents = "a,b,c,d,e,f\n1,2,3,4,5,6\n";
            let descriptor_values = vec!["1", "2", "4"];
            let value_values = vec!["0", "0", "0"];

            let expected_contents = vec!["1,2,3,4,5,6", "1,2,4,0,0,0"];
            assert_eq!(get_updated_contents(original_contents, &descriptor_values, &value_values), expected_contents);
        }

        #[test]
        fn should_sort() {
            let original_contents = r"a,b,c,d,e,f
1,2,3,_,_,_
1,1,2,_,_,_
1,2,4,_,_,_
";
            let descriptor_values = vec!["1", "2", "3"];
            let value_values = vec!["_", "_", "_"];

            let expected_contents = vec!["1,1,2,_,_,_", "1,2,3,_,_,_", "1,2,4,_,_,_"];
            assert_eq!(get_updated_contents(original_contents, &descriptor_values, &value_values), expected_contents);
        }
    }
}
