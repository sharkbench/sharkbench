
# Performance & Memory Progression Table

Using a spare laptop with an AMD Ryzen 7 6800HS (2022)

| Stage | RPS (Median) | RPS (P99) | Latency (Median) | Latency (P99) | Memory (Median) | Build Time |
|---|---|---|---|---|---|---|
| Initial WebApplication + JsonDocument | 618 | 596 | 61.27 ms | 99.65 ms | 18.07 MB | 85.93 s |
| Raw Kestrel + Utf8JsonReader (ValueSpan) | 1,145 | 1,102 | 19.62 ms | 69.79 ms | 16.52 MB | 81.35 s |
| ThreadPool.SetMinThreads & 128 KB Pool | 1,344 | 1,278 | 17.33 ms | 63.09 ms | 17.02 MB | 80.69 s |
| Remove Redundant FlushAsync() | 1,369 | 1,331 | 17.30 ms | 62.20 ms | 16.77 MB | 98.10 s |
| Dockerfile 1-Core Env Caps & Size AOT | 1,681 | 1,596 | 16.84 ms | 47.15 ms | 15.63 MB | 80.06 s |
| Inline Socket Scheduling (UnsafePreferInlineScheduling) | 1,728 | 1,642 | 16.43 ms | 46.35 ms | 15.23 MB | 83.46 s |
| Rust / Axum 0.8.9 (Reference Ceiling) | 1,860 | 1,809 | 16.86 ms | 29.02 ms | 7.83 MB | 97.50 s |

> (Overall net gain: +179.6% RPS, -73.2% median latency, -53.5% P99 latency, and -15.7% RSS—beating Rust/Axum's median latency by 0.43 ms.)