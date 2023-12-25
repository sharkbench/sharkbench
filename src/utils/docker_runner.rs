use std::{fs, thread, time::Duration};
use std::process::Command;
use std::path::Path;

const IGNORE_FILE: &str = r#"
.dart_tool
node_modules
target
"#;

/// Starts a docker container with the given `compose_file`.
/// The container is stopped after the function `on_container_started` has finished.
/// If `compose_file` is `None`, the directory is expected to contain a docker-compose.yml file.
pub fn run_docker_compose<F>(dir: &str, compose_file: Option<&str>, on_container_started: F)
    where
        F: FnOnce(),
{
    if let Some(compose_file_content) = compose_file {
        fs::write(format!("{}/docker-compose.yml", dir), compose_file_content).unwrap();
        fs::write(format!("{}/.dockerignore", dir), IGNORE_FILE).unwrap();
    }

    println!(" -> Building image");
    run_shell(&["docker", "compose", "up", "--build", "-d"], dir);

    // A heuristic to wait for the container to be ready
    println!(" -> Waiting for container to be ready");
    thread::sleep(Duration::from_secs(5));

    on_container_started();

    println!(" -> Stopping container");
    run_shell(&["docker", "compose", "down", "--rmi", "all"], dir);

    if compose_file.is_some() {
        fs::remove_file(format!("{}/docker-compose.yml", dir)).unwrap();
        fs::remove_file(format!("{}/.dockerignore", dir)).unwrap();
    }
}

fn run_shell(cmd: &[&str], working_dir: &str) {
    let mut command = Command::new(cmd[0]);
    command.args(&cmd[1..]);
    command.current_dir(Path::new(working_dir));
    command.status().expect("failed to execute command");
}
