# Sharkbench

Benchmarking programming languages and frameworks.

Checkout the results at [sharkbench.dev](https://sharkbench.dev).

## Benchmark Types

### ➤ Computation

This benchmark tests how fast a programming language can perform mathematical computations without any I/O or memory allocation.
We are using the [Leibniz formula](https://en.wikipedia.org/wiki/Leibniz_formula_for_%CF%80) to approximate the value of PI.

The program is driven over stdin/stdout, so no HTTP server or network library is involved
and the measured memory and build time only reflect the language runtime itself.

See [Computation Benchmark](#computation-benchmark) for more information.

### ➤ Memory (not yet implemented)

This benchmark tests how efficiently a programming language can perform memory management.
We are using the [A* algorithm](https://en.wikipedia.org/wiki/A*_search_algorithm), a popular pathfinding algorithm, to find the shortest path between two points.

### ➤ Web

This benchmark tests how fast a framework can perform concurrent HTTP requests, I/O operations, and JSON de/serialization.

Using [Docker](https://www.docker.com/), we are limiting the CPU usage to 1 core equivalent to not put single-threaded frameworks at a disadvantage.
Multithreaded frameworks are still able to use multiple cores but at a lower usage.

In production, single-threaded frameworks can be scaled up horizontally to use all available cores.

See [Web Framework Benchmark](#web-framework-benchmark) for more information.

## Run benchmarks

To view all available options, run:

```bash
cargo run --release -- -h
```

### ➤ Prerequisites

- [Rust](https://www.rust-lang.org/)
- [Docker](https://www.docker.com/)
- [OpenSSL](https://www.openssl.org/)

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

The version can be omitted to run all matching variants (e.g. `javascript/express-4-nodejs-12`, `javascript/express-5-bun-1`, ...):

```bash
cargo run --release -- --web --only javascript/express
```

### ➤ Missing benchmarks

Only run missing benchmarks (skipping those with existing results) by adding `--missing`:

```bash
cargo run --release -- --web --missing
```

To exit after `N` benchmarks (e.g. to let the machine cool down between runs), add `--limit N`:

```bash
cargo run --release -- --web --missing --limit 1
```

To print how many benchmarks would be run without running any, add `--count`:

```bash
cargo run --release -- --web --missing --count
```

### ➤ Prune results

Remove result rows that no longer belong to any benchmark (e.g. a renamed directory or a version removed from `benchmark.yaml`) by adding `--prune`.
It respects `--web`, `--computation`, `--lang` and `--only`. Add `--count` to only print the rows that would be removed:

```bash
cargo run --release -- --prune --count
```

## Contributing

Keep in mind that the goal of Sharkbench is to guide developers in choosing the next stack for their **production** applications.
Therefore, the submitted frameworks and programming languages should be **production-ready** and **actively maintained**.

- at least 3 months old
- evidence of production usage

There might be exceptions or additional requirements, but this is the general guideline.

For web benchmarks, always bind the server to `0.0.0.0` to allow access from outside the container.

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

## Config

Each benchmark has a `benchmark.yaml` file that contains the configuration for the benchmark.

**Minimal Example:**

```yaml
language: Java
mode: Temurin # or set "Default" if there is only one mode / flavor
version:
  - '11' # first version should match the version in the source code
  - '17'
  - '21'

# only for web benchmarks
framework: Spring Boot
framework_website: https://spring.io/projects/spring-boot
framework_flavor: MVC # or set "Default" if there is only one flavor
framework_version:
  - '2.5' # first version should match the version in the source code
  - '3.2'
```

**Complete Example:**

```yaml
language: Java
mode: Temurin # or set "Default" if there is only one mode / flavor
version:
  - '11' # first version should match the version in the source code
  - '17'
  - '21'

# specify how the version is defined in the source code
version_regex:
  Dockerfile: 'temurin[-:](\d+)'
  pom.xml: '<java\.version>(\d+)<\/java\.version>'

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
runs: 5 # override the default number of runs (ONLY for computation and memory benchmarks)

# reduce redundancy by extracting common files to the "_common" folder
copy:
  - 'pom.xml' # copy into root
  - 'application.properties': 'src/main/resources/application.properties' # copy into specific folder
```

## Computation Benchmark

The harness starts the container with stdin and stdout attached and keeps it running for all warmup and measured runs.
The program must implement the following line based protocol:

1. Read one line from stdin. It contains the number of iterations, e.g. `1000000000`.
2. Compute the three values with the Leibniz loop (see the Rust or Python benchmark for the reference implementation).
3. Write them to stdout as one line in the form `pi;sum;custom` and **flush stdout**.
4. Repeat until stdin is closed (EOF), then exit.

Example session:

```text
stdin:  1000000000
stdout: 3.1415926525880504;785398157.7092886;0.7853981633136793
```

The result is only accepted if the line contains exactly this output.

Debug output must go to stderr, since every line on stdout is interpreted as a response.

## Web Framework Benchmark

The application must listen on port `5001`.

Each benchmark has access to `http://127.0.0.1:5002/element.json` and `http://127.0.0.1:5002/shells.json`
which is provided by the [web-data-source](https://github.com/sharkbench/sharkbench/tree/main/src/benchmark/web/data/static).
This data source is used to simulate I/O (similar to database queries).

The benchmark and the data source run with host networking,
so the requests go through loopback without Docker networking in between.

The application should parse the `symbol` query parameter, fetch the json from the data source, and return the result.
The exact API is as follows:

### ➤ Route A

Request:

```text
GET /api/v1/periodic-table/element?symbol=He
```

Response:

```json
{
  "name": "Helium",
  "number": 2,
  "group": 18
}
```

### ➤ Route B

Request:

```text
GET /api/v1/periodic-table/shells?symbol=He
```

Response:

```json
{
  "shells": [2]
}
```

Both routes are called randomly. The application should be able to handle both routes concurrently.
