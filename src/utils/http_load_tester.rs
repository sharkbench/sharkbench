use crate::utils::percentile;
use crate::utils::serialization::SerializedValue;
use rand::seq::SliceRandom;
use reqwest;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::time::Duration;
use tokio;
use tokio::task::JoinHandle;
use tokio::{task, time};

pub struct HttpLoadResult {
    pub success_count: i32,
    pub fail_count: i32,
    pub total_time: Duration,
    pub rps_median: i32,
    pub rps_p99: i32,
    pub latency_median: Duration,
    pub latency_p99: Duration,
}

#[derive(Clone)]
pub struct PreparedHttpRequest {
    pub url: String,
    pub expected_response: HashMap<String, SerializedValue>,
}

struct HttpResponse<'a> {
    url: &'a str,
    body: String,
    expected_body: &'a HashMap<String, SerializedValue>,
}

type RequestValidatorFn = fn(&str, &HashMap<String, SerializedValue>) -> Result<(), String>;

pub fn run_http_load_test(
    concurrency: usize,
    duration: Duration,
    requests: &Vec<PreparedHttpRequest>,
    request_validator: RequestValidatorFn,
    verbose: bool,
) -> HttpLoadResult {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(run_load_test(
        concurrency,
        duration,
        requests,
        request_validator,
        verbose,
    ));
    result
}

async fn run_load_test(
    concurrency: usize,
    duration: Duration,
    requests: &Vec<PreparedHttpRequest>,
    request_validator: RequestValidatorFn,
    verbose: bool,
) -> HttpLoadResult {
    let mut handles: Vec<JoinHandle<ThreadResult>> = Vec::new();

    for _ in 0..concurrency {
        let mut requests_clone = requests.clone();
        requests_clone.shuffle(&mut rand::rng());

        let handle = task::spawn(async move {
            let mut local_success_count = 0;
            let mut success_count_temp = 0;
            let mut local_fail_count = 0;
            let mut local_latency_us: Vec<u64> = Vec::with_capacity(1_000_000);
            let mut rps_per_second: Vec<i32> = Vec::with_capacity(100);
            let mut responses: Vec<HttpResponse> = Vec::with_capacity(1_000_000);

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build().unwrap();
            let start = std::time::Instant::now();
            let mut second = 1;
            'outer: loop {
                for request in &requests_clone {
                    let url = &request.url;
                    let request_start = std::time::Instant::now();
                    match client.get(url).send().await {
                        Ok(response) => {
                            let status = &response.status();
                            let body = response.text().await.unwrap();
                            let latency_us = request_start.elapsed().as_micros() as u64;
                            if *status == StatusCode::OK {
                                local_success_count += 1;
                                local_latency_us.push(latency_us);
                                responses.push(HttpResponse {
                                    url,
                                    body,
                                    expected_body: &request.expected_response,
                                });
                            } else {
                                local_fail_count += 1;
                                if verbose {
                                    println!("Unexpected response {} for {}", *status, url);
                                    println!("Success: {}, Fail: {}", local_success_count, local_fail_count);
                                }
                            }
                        }
                        Err(e) => {
                            if verbose {
                                println!("Request to {} failed: {}", url, e);
                                println!("Success: {}, Fail: {}", local_success_count, local_fail_count);
                            }
                            local_fail_count += 1;
                        },
                    }

                    if start.elapsed().as_secs() >= second {
                        if second == duration.as_secs() {
                            break 'outer;
                        } else {
                            rps_per_second.push(local_success_count - success_count_temp);
                            success_count_temp = local_success_count;
                            second += 1;
                        }
                    }
                }
            }

            // validate
            for response in responses {
                let body = &response.body;
                if let Err(e) = request_validator(body, response.expected_body) {
                    local_success_count -= 1;
                    local_fail_count += 1;
                    if verbose {
                        println!("Validation failed for response for {}: {}, expected: {:?}, {}", response.url, body, response.expected_body, e);
                    }
                }
            }

            ThreadResult {
                success_count: local_success_count,
                fail_count: local_fail_count,
                latency_us: local_latency_us,
                rps_per_second,
                total_time: Duration::from_millis(start.elapsed().as_millis() as u64),
            }
        });

        handles.push(handle);
    }

    time::sleep(duration).await;

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

    if success_count == 0 {
        panic!("No successful requests. Something is wrong. Run with --verbose to see the errors.");
    }

    if fail_count > 0 {
        panic!("Some requests failed. Run with --verbose to see the errors.");
    }

    let rps_per_second: Vec<i32> = {
        let all_vectors: Vec<Vec<i32>> = handle_results.iter().map(|x| x.rps_per_second.clone()).collect();
        let mut rps_per_second: Vec<i32> = Vec::new();
        for i in 0..all_vectors[0].len() {
            let mut sum = 0;
            for vector in &all_vectors {
                sum += vector[i];
            }
            rps_per_second.push(sum);
        }
        rps_per_second.sort();
        rps_per_second
    };
    let latency_us: Vec<u64> = {
        let mut latency_us: Vec<u64> = handle_results.iter().fold(Vec::new(), |mut acc, x| {
            acc.extend(x.latency_us.clone());
            acc
        });
        latency_us.sort();
        latency_us
    };

    HttpLoadResult {
        success_count,
        fail_count,
        total_time,
        rps_median: percentile::p50(&rps_per_second),
        rps_p99: percentile::p1(&rps_per_second), // inverse because we want the worst case
        latency_median: Duration::from_micros(percentile::p50(&latency_us)),
        latency_p99: Duration::from_micros(percentile::p99(&latency_us)),
    }
}

struct ThreadResult {
    success_count: i32,
    fail_count: i32,

    /// After each second, the number of requests is written to this vector.
    /// Used to calculate P50 and P99.
    rps_per_second: Vec<i32>,

    latency_us: Vec<u64>,
    total_time: Duration,
}
