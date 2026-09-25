use std::net::IpAddr;
use std::path::Path;
use std::process::Command;
use std::{fs, thread, time::Duration};

const IGNORE_FILE: &str = r#"
.dart_tool
.gradle
build
node_modules
target
"#;

/// Starts a docker container with the given `compose_file`.
/// The container is stopped after the function `on_container_started` has finished.
/// If `compose_file` is `None`, the directory is expected to contain a docker-compose.yml file.
pub fn run_docker_compose<F>(
    dir: &str,
    delay: Duration,
    compose_file: Option<&str>,
    on_container_started: F,
) where
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
    thread::sleep(delay);

    on_container_started();

    println!(" -> Stopping container");
    // "local" only removes the images built by this project.
    // Images referenced by name (e.g. the web data source) are kept for the next benchmark.
    run_shell(&["docker", "compose", "down", "--rmi", "local"], dir);

    if compose_file.is_some() {
        fs::remove_file(format!("{}/docker-compose.yml", dir)).unwrap();
        fs::remove_file(format!("{}/.dockerignore", dir)).unwrap();
    }
}

/// Builds the images of the docker-compose.yml file in `dir` without starting any container.
/// The images are removed after the function `on_images_built` has finished.
pub fn build_docker_compose<F>(dir: &str, on_images_built: F)
where
    F: FnOnce(),
{
    println!(" -> Building image");
    run_shell(&["docker", "compose", "build"], dir);

    on_images_built();

    println!(" -> Removing image");
    run_shell(&["docker", "compose", "down", "--rmi", "all"], dir);
}

/// Returns the IP address of the container and the gateway of its Docker network.
pub fn get_container_network(container_name: &str) -> Option<(IpAddr, IpAddr)> {
    let output = Command::new("docker")
        .args([
            "inspect",
            "--format",
            "{{range .NetworkSettings.Networks}}{{.IPAddress}} {{.Gateway}} {{end}}",
            container_name,
        ])
        .output()
        .ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;
    let mut ips = stdout.split_whitespace().map(|ip| ip.parse::<IpAddr>());
    match (ips.next(), ips.next()) {
        (Some(Ok(ip)), Some(Ok(gateway))) => Some((ip, gateway)),
        _ => None,
    }
}

fn run_shell(cmd: &[&str], working_dir: &str) {
    let mut command = Command::new(cmd[0]);
    command.args(&cmd[1..]);
    command.current_dir(Path::new(working_dir));
    let status = command
        .status()
        .expect(&format!("failed to execute command: {:?}", cmd));
    if !status.success() {
        panic!("Command failed: {:?}", cmd);
    }
}
