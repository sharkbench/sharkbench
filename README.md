# Sharkbench

Benchmarking programming languages and frameworks.

Checkout the results at [sharkbench.dev](https://sharkbench.dev).

## Benchmark Types

### ➤ Computation Benchmark

This benchmark tests how fast a programming language can perform mathematical computations without any I/O or memory allocation.
We are using the [Leibniz formula](https://en.wikipedia.org/wiki/Leibniz_formula_for_%CF%80) to approximate the value of PI.

### ➤ Memory Benchmark

This benchmark tests how efficiently a programming language can perform memory management.
We are using the [A* algorithm](https://en.wikipedia.org/wiki/A*_search_algorithm), a popular pathfinding algorithm, to find the shortest path between two points.

### ➤ Web Framework Benchmark

This benchmark tests how fast a framework can perform concurrent HTTP requests, I/O operations, and JSON de/serialization.

Using [Docker](https://www.docker.com/), we are limiting the CPU usage to 1 core equivalent to not put single-threaded frameworks at a disadvantage.
Multithreaded frameworks are still able to use multiple cores but at a lower usage.

In production, single-threaded frameworks can be scaled up horizontally to use all available cores.

Request:

```text
GET /api/v1/periodic-table?element=He
```

Response:

```json
{
  "name": "Helium",
  "number": 2,
  "group": 18
}
```

## Run benchmarks

To view all available options, run:

```bash
cargo run --release -- -h
```

### ➤ Prerequisites

- [Rust](https://www.rust-lang.org/)
- [Docker](https://www.docker.com/)

Create Docker network:

```bash
docker network create sharkbench-benchmark-network
```

### ➤ Run all benchmarks

To run all benchmarks, run:

```bash
cargo run --release
```

### ➤ Specific benchmark type

To run only one benchmark type, add `--web`, `--computation`, or `--memory`:

```bash
cargo run --release -- --web
```

### ➤ Specific benchmark programming language

Limit the programming languages to run by adding `--lang <language>`:

```bash
cargo run --release -- --web --lang java
```

### ➤ Specific benchmark

Only run a specific benchmark by adding `--only <benchmark>`:

```bash
cargo run --release -- --web --only javascript/express-4-nodejs-12
```

## Contributing

### ➤ File structure

In general, each benchmark is located in a separate folder.

- `benchmark/`: Contains all benchmarks.
  - `computation/`: Contains all computation benchmarks.
    - `<language>/<mode>-<min-version>`: A benchmark.
  - `memory/`: Contains all memory benchmarks.
    - `<language>/<mode>-<min-version>`: A benchmark.
  - `web/`: Contains all web benchmarks.
    - `<language>/<framework>-<min-framework-version>-<mode>-<min-version>`: A benchmark.
- `src/`: The main source code to run the benchmarks.

### ➤ Config

Each benchmark has a `benchmark.yaml` file that contains the configuration for the benchmark.

```yaml
language: Java
mode: Temurin # or set "Default" if there is only one mode / flavor
version:
  - '11' # first version should match the version in the source code
  - '17'
  - '21'

# only for web benchmarks
framework: Spring Boot
framework_stdlib: false # OPTIONAL: set to true if the framework is part of the standard library
framework_website: https://spring.io/projects/spring-boot
framework_flavor: MVC # or set "Default" if there is only one flavor
framework_version:
  - '2.5' # first version should match the version in the source code
  - '3.2'

# optional
extended_warmup: true # set to true if the benchmark needs a longer warmup
concurrency: 4 # override the default concurrency
```
