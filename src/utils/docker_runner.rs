use regex::Regex;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Instant;
use std::{fs, thread, time::Duration};

/// Name of the benchmark container, also used to filter `docker stats`.
/// The compose files of the benchmarks must use the same `container_name`.
pub const CONTAINER_NAME: &str = "benchmark";

/// Name of the service in the compose files of the benchmarks.
const SERVICE_NAME: &str = "benchmark";

/// How long to wait for a detached container to accept connections on its port.
const READY_TIMEOUT: Duration = Duration::from_secs(120);

/// Minimum time to wait for a detached container, even if its port is open earlier,
/// so that servers which open the port before finishing their initialization can settle.
const MIN_READY_DELAY: Duration = Duration::from_secs(5);

/// Image used to drop the page cache of the host, see [`drop_page_cache`].
const DROP_CACHE_IMAGE: &str = "alpine:3.22";

/// Directories excluded from the docker context.
pub const IGNORED_DIRS: [&str; 5] = [".dart_tool", ".gradle", "build", "node_modules", "target"];

/// How the benchmark container is started.
pub enum StartMode {
    /// `docker compose up -d`. The harness talks to the container over the network.
    /// If `ready_port` is set, the harness waits until 127.0.0.1:`ready_port`
    /// accepts connections (but at least [`MIN_READY_DELAY`]) before doing so.
    Detached { ready_port: Option<u16> },

    /// `docker compose run` with the container's stdin and stdout piped to the harness.
    /// The container is expected to exit when its stdin is closed.
    Attached,
}

/// A running container whose stdin and stdout are connected to the harness.
///
/// The protocol is line based: [`AttachedContainer::write_line`] sends one request,
/// [`AttachedContainer::read_line`] waits for one response line.
pub struct AttachedContainer {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<std::io::Result<String>>,
}

impl AttachedContainer {
    fn spawn(dir: &str) -> AttachedContainer {
        let mut child = Command::new("docker")
            .args(&[
                "compose",
                "run",
                "--rm",
                "--no-deps",
                "-T",
                "--name",
                CONTAINER_NAME,
                SERVICE_NAME,
            ])
            .current_dir(Path::new(dir))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start container");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        // Read stdout on a separate thread so that reads can time out.
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });

        AttachedContainer {
            child,
            stdin: Some(stdin),
            lines: receiver,
        }
    }

    /// Sends one line (without trailing newline) to the container's stdin.
    pub fn write_line(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or("Container stdin is already closed")?;
        stdin.write_all(line.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
        Ok(())
    }

    /// Waits for the next line printed on the container's stdout.
    /// Fails if the container exits or does not answer within `timeout`.
    pub fn read_line(&mut self, timeout: Duration) -> Result<String, Box<dyn std::error::Error>> {
        match self.lines.recv_timeout(timeout) {
            Ok(Ok(line)) => Ok(line),
            Ok(Err(e)) => Err(Box::from(format!("Failed to read from container: {e}"))),
            Err(RecvTimeoutError::Timeout) => Err(Box::from(format!(
                "No response from container within {} s",
                timeout.as_secs()
            ))),
            Err(RecvTimeoutError::Disconnected) => Err(Box::from(
                "Container closed its stdout (the process probably exited)",
            )),
        }
    }

    /// Closes stdin and waits for the container to exit.
    /// Containers that ignore EOF are removed by force.
    fn finish(mut self, dir: &str) {
        drop(self.stdin.take());

        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(100)),
                Ok(None) => break,
                Err(e) => panic!("Failed to wait for container: {e}"),
            }
        }

        println!(" -> Container did not exit after stdin was closed, removing it");
        let _ = Command::new("docker")
            .args(&["rm", "-f", CONTAINER_NAME])
            .current_dir(Path::new(dir))
            .status();
        let _ = self.child.wait();
    }
}

/// Starts a docker container with the given `compose_file`.
/// The container is stopped after the function `on_container_started` has finished.
/// If `compose_file` is `None`, the directory is expected to contain a docker-compose.yml file.
///
/// `on_container_started` receives the attached container in [`StartMode::Attached`]
/// and `None` in [`StartMode::Detached`].
///
/// If `measure_build` is true, the base images are pulled beforehand and the image is built
/// without cache. The returned duration is the build time, excluding base image downloads
/// and container startup. Otherwise, `None` is returned.
pub fn run_docker_compose<F>(
    dir: &str,
    start_mode: StartMode,
    compose_file: Option<&str>,
    measure_build: bool,
    on_container_started: F,
) -> Option<Duration>
where
    F: FnOnce(Option<&mut AttachedContainer>),
{
    if let Some(compose_file_content) = compose_file {
        fs::write(format!("{}/docker-compose.yml", dir), compose_file_content).unwrap();
        fs::write(format!("{}/.dockerignore", dir), IGNORED_DIRS.join("\n")).unwrap();
    }

    let build_time = if measure_build {
        // Expects the Dockerfile to be already migrated to the target version
        pull_base_images(dir);

        // Cold build, independent of previously pulled images and earlier tasks
        drop_page_cache(dir);

        println!(" -> Building image");
        let start = Instant::now();
        run_shell(&["docker", "compose", "build", "--no-cache"], dir);
        let build_time = start.elapsed();
        println!(" -> Build time: {} ms", build_time.as_millis());
        Some(build_time)
    } else {
        println!(" -> Building image");
        run_shell(&["docker", "compose", "build"], dir);
        None
    };

    drop_page_cache(dir);

    println!(" -> Starting container");
    match start_mode {
        StartMode::Detached { ready_port } => {
            run_shell(&["docker", "compose", "up", "-d"], dir);

            if let Some(port) = ready_port {
                println!(" -> Waiting for container to be ready");
                wait_for_port(dir, port);
            }

            on_container_started(None);
        }
        StartMode::Attached => {
            // No readiness heuristic needed: the first request waits in the pipe
            // until the process reads it.
            let mut container = AttachedContainer::spawn(dir);
            on_container_started(Some(&mut container));
            container.finish(dir);
        }
    }

    println!(" -> Stopping container");
    run_shell(&["docker", "compose", "down", "--rmi", "all"], dir);

    if compose_file.is_some() {
        fs::remove_file(format!("{}/docker-compose.yml", dir)).unwrap();
        fs::remove_file(format!("{}/.dockerignore", dir)).unwrap();
    }

    build_time
}

/// Blocks until 127.0.0.1:`port` accepts a TCP connection.
/// Waits at least [`MIN_READY_DELAY`] in total.
/// Panics if this does not happen within [`READY_TIMEOUT`].
fn wait_for_port(dir: &str, port: u16) {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let start = Instant::now();
    while TcpStream::connect_timeout(&address, Duration::from_secs(1)).is_err() {
        if start.elapsed() > READY_TIMEOUT {
            panic!(
                "[{dir}] Container did not accept connections on port {port} within {} s",
                READY_TIMEOUT.as_secs()
            );
        }
        thread::sleep(Duration::from_millis(100));
    }
    println!(" -> Ready after {} ms", start.elapsed().as_millis());
    if let Some(remaining) = MIN_READY_DELAY.checked_sub(start.elapsed()) {
        thread::sleep(remaining);
    }
}

/// Drops the page cache of the host so that every build and container starts cold.
/// Keeps build time and memory (RAM) stats consistent.
fn drop_page_cache(dir: &str) {
    println!(" -> Dropping page cache");
    run_shell(&["sync"], dir);
    run_shell(
        &[
            "docker",
            "run",
            "--rm",
            "--privileged",
            DROP_CACHE_IMAGE,
            "sh",
            "-c",
            "echo 3 > /proc/sys/vm/drop_caches",
        ],
        dir,
    );
}

/// Pulls the base images of the Dockerfile in `dir`
/// so that the download is not part of the measured build time.
fn pull_base_images(dir: &str) {
    let dockerfile = fs::read_to_string(format!("{}/Dockerfile", dir)).unwrap();
    for image in parse_base_images(&dockerfile) {
        println!(" -> Pulling {}", image);
        run_shell(&["docker", "pull", image.as_str()], dir);
    }
}

/// Returns the external images referenced by `FROM` instructions.
/// Skips `scratch` and references to earlier build stages.
/// Variables are resolved with the defaults of global `ARG` instructions.
fn parse_base_images(dockerfile: &str) -> Vec<String> {
    let variable_regex = Regex::new(r"\$\{?(\w+)\}?").unwrap();
    let mut global_args: HashMap<String, String> = HashMap::new();
    let mut stages: Vec<String> = Vec::new();
    let mut images: Vec<String> = Vec::new();
    let mut seen_from = false;

    for line in dockerfile.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let Some(instruction) = parts.first() else {
            continue;
        };

        // Only ARGs declared before the first FROM can be used in FROM
        if instruction.eq_ignore_ascii_case("ARG") && !seen_from {
            if let Some((name, value)) = parts.get(1).and_then(|p| p.split_once('=')) {
                global_args.insert(name.to_string(), value.trim_matches('"').to_string());
            }
            continue;
        }

        if !instruction.eq_ignore_ascii_case("FROM") {
            continue;
        }
        seen_from = true;

        // Skip flags like --platform=...
        let Some(image) = parts.iter().skip(1).find(|p| !p.starts_with("--")) else {
            continue;
        };

        let mut resolved = true;
        let image = variable_regex
            .replace_all(image, |caps: &regex::Captures| {
                global_args.get(&caps[1]).cloned().unwrap_or_else(|| {
                    resolved = false;
                    String::new()
                })
            })
            .to_string();

        let is_stage = stages.iter().any(|s| s.eq_ignore_ascii_case(&image));
        if resolved && !is_stage && image != "scratch" && !images.contains(&image) {
            images.push(image);
        }

        if let Some(i) = parts.iter().position(|p| p.eq_ignore_ascii_case("AS")) {
            if let Some(stage) = parts.get(i + 1) {
                stages.push(stage.to_string());
            }
        }
    }

    images
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

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_base_images {
        use super::*;

        #[test]
        fn should_parse_single_image() {
            let dockerfile = "FROM node:22-alpine\nCOPY . .\nCMD [\"node\", \"index.js\"]\n";
            assert_eq!(parse_base_images(dockerfile), vec!["node:22-alpine"]);
        }

        #[test]
        fn should_skip_stages_and_scratch() {
            let dockerfile = r"FROM rust:1.87 AS Builder
RUN cargo build --release

FROM builder AS intermediate

FROM scratch
COPY --from=builder /app /app

FROM debian:13-slim
";
            assert_eq!(
                parse_base_images(dockerfile),
                vec!["rust:1.87", "debian:13-slim"]
            );
        }

        #[test]
        fn should_skip_flags() {
            let dockerfile = "FROM --platform=linux/amd64 golang:1.24 as build\n";
            assert_eq!(parse_base_images(dockerfile), vec!["golang:1.24"]);
        }

        #[test]
        fn should_resolve_global_args() {
            let dockerfile = r#"ARG JAVA_VERSION=21
ARG RUBY_VERSION="3.2"
FROM maven:3-eclipse-temurin-${JAVA_VERSION} AS build
FROM ruby:$RUBY_VERSION-slim
FROM eclipse-temurin:${JAVA_VERSION}
"#;
            assert_eq!(
                parse_base_images(dockerfile),
                vec![
                    "maven:3-eclipse-temurin-21",
                    "ruby:3.2-slim",
                    "eclipse-temurin:21"
                ]
            );
        }

        #[test]
        fn should_ignore_stage_args_and_unresolved_variables() {
            let dockerfile = r"FROM alpine:3.21
ARG VERSION=1
FROM node:${VERSION}
FROM python:${UNKNOWN}
";
            assert_eq!(parse_base_images(dockerfile), vec!["alpine:3.21"]);
        }

        #[test]
        fn should_deduplicate_images() {
            let dockerfile = "FROM alpine:3.21 AS a\nFROM alpine:3.21 AS b\n";
            assert_eq!(parse_base_images(dockerfile), vec!["alpine:3.21"]);
        }
    }
}
