# Sharkbench

Benchmarking web frameworks and languages.

## Run benchmarks

```bash
dart pub get
dart run benchmark.dart
```

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
