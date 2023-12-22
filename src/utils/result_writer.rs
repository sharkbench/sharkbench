use std::{fs, io};

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

    let descriptor_values: String = descriptors.iter().map(|(_, v)| *v).collect::<Vec<&str>>().join(",");
    let values_values: String = values.iter().map(|(_, v)| *v).collect::<Vec<&str>>().join(",");

    let original_contents: String = fs::read_to_string(file_path).unwrap_or(String::new());

    let mut contents: Vec<String> = original_contents
        .lines()
        .skip(1)
        .filter(|line| !line.starts_with(&descriptor_values))
        .map(|line| line.to_string())
        .collect::<Vec<String>>();

    contents.push(format!("{},{}", descriptor_values, values_values));
    contents.sort();

    let mut output = String::new();

    // Header
    let descriptor_keys: String = descriptors.iter().map(|(k, _)| *k).collect::<Vec<&str>>().join(",");
    let values_keys: String = values.iter().map(|(k, _)| *k).collect::<Vec<&str>>().join(",");
    output.push_str(descriptor_keys.as_str());
    output.push(',');
    output.push_str(values_keys.as_str());
    output.push('\n');

    // Data
    for line in contents {
        output.push_str(&line);
        output.push_str("\n");
    }

    fs::write(file_path, output.as_bytes()).expect(format!("Failed to write {}", file_path).as_str());

    Ok(())
}
