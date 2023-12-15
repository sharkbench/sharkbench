use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn hello() {
    // Shared data structure for storing the output
    let output_list = Arc::new(Mutex::new(Vec::new()));

    // Clone the Arc to move into the thread
    let output_list_clone = Arc::clone(&output_list);

    // Spawn a new thread to run the command
    thread::spawn(move || {
        let mut command = Command::new("docker");
        command.args(["stats", "--format", "json"]);
        command.stdout(Stdio::piped());

        let mut child = command.spawn().expect("Failed to start docker command");
        let stdout = child.stdout.take().expect("Failed to capture stdout");

        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line.expect("Failed to read line");

            // Store each line in the shared data structure
            let mut list = output_list_clone.lock().unwrap();

            println!("{}", line);

            list.push(line);
        }
    });
}
