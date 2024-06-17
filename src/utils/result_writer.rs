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
///
/// `on_conflict` is a function that is called when a line with the same descriptor values already exists.
/// It receives the existing values and the new values and should return the values that should be written to the file.
pub fn write_result_to_file(
    file_path: &str,
    descriptors: &Vec<(&str, &str)>,
    values: &Vec<(&str, &str)>,
    on_conflict: for<'a> fn(&'a [&'a str], &'a [&'a str]) -> &'a [&'a str],
) -> io::Result<()> {
    println!(" -> Writing result:");
    for (key, value) in descriptors {
        println!("    - {}: {}", key, value);
    }
    for (key, value) in values {
        println!("    - {}: {}", key, value);
    }

    let header: String = {
        let descriptor_keys: String = descriptors.iter().map(|(k, _)| *k).collect::<Vec<&str>>().join(",");
        let values_keys: String = values.iter().map(|(k, _)| *k).collect::<Vec<&str>>().join(",");
        format!("{},{}", descriptor_keys, values_keys)
    };

    let old_contents = fs::read_to_string(file_path).unwrap_or(String::new());

    let descriptor_values: Vec<&str> = descriptors.iter().map(|(_, v)| *v).collect::<Vec<&str>>();
    let value_values: Vec<&str> = values.iter().map(|(_, v)| *v).collect::<Vec<&str>>();

    let contents: Vec<String> = {
        if old_contents.is_empty() {
            vec!(format!("{},{}", descriptor_values.join(","), value_values.join(",")))
        } else {
            get_updated_contents(&old_contents, descriptor_values.as_slice(), value_values.as_slice(), on_conflict)
        }
    };

    write_lines_to_file(file_path, &contents, &header).unwrap();

    Ok(())
}

fn get_updated_contents(
    old_contents: &str,
    descriptor_values: &[&str],
    value_values: &[&str],
    on_conflict: for<'a> fn(&'a [&'a str], &'a [&'a str]) -> &'a [&'a str],
) -> Vec<String> {
    let old_lines: Vec<Vec<&str>> = old_contents
        .lines()
        .skip(1)
        .map(|line| line.split(",").collect())
        .collect();

    let new_lines: Vec<Vec<&str>> = {
        let mut temp_lines: Vec<Vec<&str>> = Vec::new();
        let mut found = false;

        for columns in &old_lines {
            if columns.starts_with(descriptor_values) {
                found = true;
                let mut new_line: Vec<&str> = Vec::new();
                for i in 0..descriptor_values.len() {
                    new_line.push(descriptor_values[i]);
                }

                let old_values: &[&str] = &columns[descriptor_values.len()..columns.len()];
                let result: &[&str] = on_conflict(old_values, value_values);
                for value in result {
                    new_line.push(&value);
                }
                temp_lines.push(new_line);
            } else {
                temp_lines.push(columns.clone());
            }
        }

        if !found {
            let mut new_line = Vec::new();
            new_line.extend_from_slice(descriptor_values);
            new_line.extend_from_slice(value_values);
            temp_lines.push(new_line);
        }

        temp_lines.sort_by(compare_lines);
        temp_lines
    };

    new_lines
        .iter()
        .map(|line| line.join(","))
        .collect()
}

fn write_lines_to_file(file_path: &str, lines: &Vec<String>, header: &String) -> io::Result<()> {
    let output: String = {
        let mut buffer = String::new();

        // Header
        buffer.push_str(header);
        buffer.push('\n');

        // Data
        for line in lines {
            buffer.push_str(&line);
            buffer.push('\n');
        }

        buffer
    };

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
                    Ordering::Equal => continue, // check next column
                    ordering => return ordering,
                }
            }
            _ => match a_value.cmp(&b_value) {
                Ordering::Equal => continue, // check next column
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

        fn take_new_line<'a>(_: &[&'a str], new_values: &'a [&'a str]) -> &'a [&'a str] {
            new_values
        }

        fn take_bigger_number<'a>(old_values: &'a [&'a str], new_values: &'a [&'a str]) -> &'a [&'a str] {
            if old_values[0].parse::<i32>().unwrap() < new_values[0].parse::<i32>().unwrap() {
                new_values
            } else {
                old_values
            }
        }

        #[test]
        fn should_update_single_line() {
            let original_contents = "a,b,c,d,e,f\n1,2,3,4,5,6\n";
            let descriptor_values = vec!["1", "2", "3"];
            let value_values = vec!["0", "0", "0"];

            let expected_contents = vec!["1,2,3,0,0,0"];
            assert_eq!(get_updated_contents(original_contents, descriptor_values.as_slice(), value_values.as_slice(), take_new_line), expected_contents);
        }

        #[test]
        fn should_take_bigger_number() {
            let original_contents = "a,b,c\n1,2,3,10,20\n";
            let descriptor_values = vec!["1", "2", "3"];

            let value_values = vec!["5", "20"];
            let expected_contents = vec!["1,2,3,10,20"];
            assert_eq!(get_updated_contents(original_contents, descriptor_values.as_slice(), value_values.as_slice(), take_bigger_number), expected_contents);

            let value_values = vec!["15", "20"];
            let expected_contents = vec!["1,2,3,15,20"];
            assert_eq!(get_updated_contents(original_contents, descriptor_values.as_slice(), value_values.as_slice(), take_bigger_number), expected_contents);
        }

        #[test]
        fn should_add_new_line_if_not_exists() {
            let original_contents = "a,b,c,d,e,f\n1,2,3,4,5,6\n";
            let descriptor_values = vec!["1", "2", "4"];
            let value_values = vec!["0", "0", "0"];

            let expected_contents = vec!["1,2,3,4,5,6", "1,2,4,0,0,0"];
            assert_eq!(get_updated_contents(original_contents, descriptor_values.as_slice(), value_values.as_slice(), take_new_line), expected_contents);
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
            assert_eq!(get_updated_contents(original_contents, descriptor_values.as_slice(), value_values.as_slice(), take_new_line), expected_contents);
        }
    }
}
