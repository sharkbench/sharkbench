use std::time::Duration;

const COMPOSE_FILE: &str = r#"
version: "3"
services:
  benchmark:
    build: .
    container_name: benchmark
    ports:
      - "3000:3000"
    sysctls:
      - net.ipv4.ip_local_port_range=1024 65535

networks:
  default:
    name: "sharkbench-benchmark-network"
    external: true
"#;

pub fn run_benchmark() {
    let dir = "benchmark/web/dart/httpserver-aot-2.14";

    crate::utils::docker_runner::run_docker_compose(dir, Option::from(COMPOSE_FILE), || {
        println!("Container started");
        std::thread::sleep(Duration::from_secs(5));
    });
}
