# Sharkbench

Benchmarking programming languages and frameworks.

## Computation Benchmark

Here, we are testing how fast a programming language (or its implementation)
can approximate the value of PI using the [Leibniz formula](https://en.wikipedia.org/wiki/Leibniz_formula_for_%CF%80).

This benchmark tests how fast a language can perform
mathematical computations without any I/O or memory allocation.

## Memory Benchmark

Here, we are testing how fast a programming language (or its implementation)
can solve memory heavy tasks.
(TODO: Find a good benchmark for this.)

This benchmark tests how efficient a language can perform
memory management (i.e., memory allocation, memory deallocation, garbage collection)

## Web Framework Benchmark

Here, we are testing how fast a web framework can serve a simple JSON response.
It needs to parse the request, wait for an I/O operation, and serialize the response.

This benchmark tests how fast a web framework can perform
I/O and JSON serialization / deserialization.

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

### ➤ Prerequisites

- [Rust](https://www.rust-lang.org/)
- [Docker](https://www.docker.com/)

### ➤ Batch

To run all benchmarks, run:

```bash
cargo run --release
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

Each benchmark has a `_benchmark.yaml` file that contains the configuration for the benchmark.

```yaml
language: Dart
mode: AOT # or set "Default" if there is only one run mode / flavor
version:
  - "2.14" # first version should match the version in the Dockerfile
  - "3.2"

# only for web benchmarks
framework: Shelf
framework_version:
  - "1" # first version should match the version in the Dockerfile
  - "2"
```
