use std::collections::HashMap;
use reqwest;
use tokio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;
use tokio::{task, time};
use rand::seq::SliceRandom;
use tokio::task::JoinHandle;
use crate::utils::serialization::SerializedValue;

pub struct HttpLoadResult {
    pub success_count: i32,
    pub fail_count: i32,
    pub total_time: Duration,
    pub requests_per_second: i32,
    pub latency_median: Duration,
}

type RequestValidatorFn = fn(&str, &HashMap<String, SerializedValue>) -> bool;

pub fn run_http_load_test(
    concurrency: usize,
    duration: Duration,
    requests: &Vec<(String, HashMap<String, SerializedValue>)>,
    request_validator: RequestValidatorFn,
) -> HttpLoadResult {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(run_load_test(
        concurrency,
        duration,
        requests,
        request_validator,
    ));
    result
}

async fn run_load_test(
    concurrency: usize,
    duration: Duration,
    requests: &Vec<(String, HashMap<String, SerializedValue>)>,
    request_validator: RequestValidatorFn,
) -> HttpLoadResult {
    let mut handles: Vec<JoinHandle<ThreadResult>> = Vec::new();

    let running = Arc::new(AtomicBool::new(true));

    for _ in 0..concurrency {
        let mut requests_clone = requests.clone();
        requests_clone.shuffle(&mut rand::thread_rng());
        let running_clone = running.clone();

        let handle = task::spawn(async move {
            let mut local_success_count = 0;
            let mut local_fail_count = 0;
            let mut local_latency_us: Vec<u64> = Vec::new();
            local_latency_us.reserve(100000);

            let client = reqwest::Client::builder().build().unwrap();
            let start = std::time::Instant::now();
            'outer: loop {
                for (uri, expected_response) in &requests_clone {
                    if !running_clone.load(Relaxed) {
                        break 'outer;
                    }

                    let request_start = std::time::Instant::now();
                    match client.get(uri).send().await {
                        Ok(response) => {
                            let body = response.text().await.unwrap();
                            let latency_us = request_start.elapsed().as_micros() as u64;
                            if request_validator(&body, &expected_response) {
                                local_success_count += 1;
                                local_latency_us.push(latency_us);
                            } else {
                                local_fail_count += 1;
                                println!("Unexpected response for {}: {}, expected: {:?}", uri, body, expected_response);
                            }
                        }
                        Err(e) => {
                            println!("Request to {} failed: {}", uri, e);
                            local_fail_count += 1;
                        },
                    }
                }
            }

            ThreadResult {
                success_count: local_success_count,
                fail_count: local_fail_count,
                latency_us: local_latency_us,
                total_time: Duration::from_millis(start.elapsed().as_millis() as u64),
            }
        });

        handles.push(handle);
    }

    time::sleep(duration).await;

    // Cancel all running tasks after the duration
    running.store(false, Relaxed);

    let mut handle_results: Vec<ThreadResult> = Vec::new();
    for handle in handles.into_iter() {
        handle_results.push(handle.await.unwrap());
    }

    // max time of all threads
    let total_time = handle_results.iter().fold(Duration::from_secs(0), |acc, x| {
        if x.total_time > acc {
            x.total_time
        } else {
            acc
        }
    });

    let success_count = handle_results.iter().fold(0, |acc, x| acc + x.success_count);
    let fail_count = handle_results.iter().fold(0, |acc, x| acc + x.fail_count);

    HttpLoadResult {
        success_count,
        fail_count,
        total_time,
        requests_per_second: (success_count as f64 / total_time.as_secs_f64()) as i32,
        latency_median: {
            let mut latency_us: Vec<u64> = handle_results.iter().fold(Vec::new(), |mut acc, x| {
                acc.extend(x.latency_us.clone());
                acc
            });
            latency_us.sort();
            Duration::from_micros(latency_us[latency_us.len() / 2])
        },
    }
}

struct ThreadResult {
    success_count: i32,
    fail_count: i32,
    latency_us: Vec<u64>,
    total_time: Duration,
}
