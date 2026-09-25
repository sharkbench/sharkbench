use crate::benchmark::benchmark::{run_benchmark, AdditionalData, IterationResult};
use crate::utils::copy_files;
use crate::utils::docker_runner::get_container_network;
use crate::utils::docker_stats::DockerStatsReader;
use crate::utils::http_load_tester::{
    run_http_load_test, PendingValidationResponse, PreparedHttpRequest,
};
use crate::utils::meta_data_parser::WebBenchmarkMetaData;
use crate::utils::result_reader::ExistingResult;
use crate::utils::result_writer::write_result_to_file;
use crate::utils::serialization::SerializedValue;
use crate::utils::version_migrator::VersionMigrator;
use indexmap::IndexMap;
use serde::Deserialize;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::fs;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

/// The web data source runs as a sidecar sharing the network namespace of the benchmark
/// (like the containers of a Kubernetes pod) and `web-data-source` resolves to 127.0.0.1.
/// The data is fetched via loopback instead of the Docker network (no DNS lookup, no bridge),
/// so the benchmark measures the framework instead of the network noise.
/// Therefore, port 80 is reserved for the data source and must not be used by the benchmark.
/// The data source is published on port 3001 to reset its request counter.
/// Clients that ignore /etc/hosts and only use DNS (e.g. rama) get the network alias instead,
/// the IP of the benchmark container, so they fetch the data locally too.
const COMPOSE_FILE: &str = r#"
services:
  benchmark:
    build: .
    container_name: benchmark
    ports:
      - "3000:3000"
      - "3001:80"
    extra_hosts:
      - "web-data-source:127.0.0.1"
    networks:
      default:
        aliases:
          - web-data-source
    sysctls:
      - net.ipv4.ip_local_port_range=1024 65535
    deploy:
      resources:
        limits:
          cpus: "1.0"

  web-data-source:
    image: sharkbench-web-data-source
    pull_policy: never
    container_name: web_data_source
    network_mode: "service:benchmark"
    # The data source ignores SIGTERM, without init each compose down waits 10 seconds
    init: true

networks:
  default:
    name: "sharkbench-benchmark-network"
    external: true
"#;

const DEFAULT_CONCURRENCY: usize = 32;

pub fn benchmark_web(
    dir: &str,
    existing: Option<&ExistingResult>,
    stats_reader: &mut DockerStatsReader,
    validate: bool,
    verbose: bool,
) {
    let meta_data: WebBenchmarkMetaData = WebBenchmarkMetaData::read_from_directory(dir)
        .expect(&format!("Failed to read meta data: {dir}"));

    // Early check if all existing results are in metadata to avoid printing metadata info
    if let Some(existing) = existing {
        if meta_data.language_version.iter().all(|lang_version| {
            meta_data.framework_version.iter().all(|framework_version| {
                existing.language_versions.contains(lang_version)
                    && existing.framework_versions.contains(framework_version)
            })
        }) {
            println!(" -> Skipping {dir}");
            return;
        }
    }

    println!(" -> Benchmarking {dir}");
    meta_data.print_info();

    let data: HashMap<String, PeriodicTableElement> = load_data();

    let concurrency = match meta_data.concurrency {
        Some(concurrency) => {
            println!(
                " -> Using concurrency = {} instead of default = {}",
                concurrency, DEFAULT_CONCURRENCY
            );
            concurrency
        }
        None => DEFAULT_CONCURRENCY,
    };

    for language_version in &meta_data.language_version {
        for framework_version in &meta_data.framework_version {
            if let Some(existing) = existing {
                if existing.language_versions.contains(language_version)
                    && existing.framework_versions.contains(framework_version)
                {
                    println!(
                        " -> Skipping {} v{} / {} v{} (already exists)",
                        meta_data.mode, language_version, meta_data.framework, framework_version
                    );
                    continue;
                }
            }

            if let Some(copy_files) = &meta_data.copy {
                copy_files::copy_files(dir, &copy_files);
            }

            let mut version_migrations = Vec::with_capacity(2);

            if meta_data.language_version.len() > 1 {
                version_migrations.push(VersionMigrator::new(
                    dir,
                    meta_data.language_version_regex.clone(),
                    meta_data.language_version[0].clone(),
                    language_version.clone(),
                ));
            }

            if meta_data.framework_version.len() > 1 {
                // Also migrate the framework version
                version_migrations.push(VersionMigrator::new(
                    dir,
                    meta_data.framework_version_regex.clone(),
                    meta_data.framework_version[0].clone(),
                    framework_version.clone(),
                ));
            }

            // Prepared once the container is started, as the URL depends on the container
            let requests: OnceCell<Vec<PreparedHttpRequest>> = OnceCell::new();

            #[rustfmt::skip]
            let result = run_benchmark(
                dir,
                COMPOSE_FILE,
                stats_reader,
                version_migrations.iter_mut().collect(),
                match validate {
                    true => 0,
                    false => match meta_data.extended_warmup {
                        true => 5,
                        false => 1,
                    },
                },
                match validate {
                    true => 1,
                    false => 5,
                },
                || {
                    let requests = requests.get_or_init(|| prepare_requests(&get_benchmark_url(), &data));

                    reset_data_source_counter();

                    let result = run_http_load_test(
                        concurrency,
                        Duration::from_secs(match validate {
                            true => 2,
                            false => 15,
                        }),
                        requests,
                        response_validator,
                        verbose,
                    );

                    let data_source_counter = reset_data_source_counter();
                    if data_source_counter < result.success_count {
                        // Note: data_source_counter might be bigger when some requests are timed out, which is fine
                        panic!("Request count measured by data source: {}.
Successful responses by framework: {}.
Maybe some requests were not fired but cached responses were used?",
                            data_source_counter, result.success_count);
                    }

                    let mut additional_data: IndexMap<String, AdditionalData> = IndexMap::new();
                    additional_data.insert("rps_median".to_string(), AdditionalData::Int(result.rps_median));
                    additional_data.insert("rps_p99".to_string(), AdditionalData::Int(result.rps_p99));
                    additional_data.insert("latency_median".to_string(), AdditionalData::Int(result.latency_median.as_micros() as i32));
                    additional_data.insert("latency_p99".to_string(), AdditionalData::Int(result.latency_p99.as_micros() as i32));
                    additional_data.insert("errors".to_string(), AdditionalData::Int(result.fail_count));

                    let mut debugging_data: IndexMap<String, AdditionalData> = IndexMap::new();
                    debugging_data.insert("success".to_string(), AdditionalData::Int(result.success_count));
                    debugging_data.insert("time".to_string(), AdditionalData::Int(result.total_time.as_millis() as i32));

                    Ok(IterationResult {
                        additional_data,
                        debugging_data,
                    })
                },
            );

            if let Some(copy_files) = &meta_data.copy {
                copy_files::delete_copied_files(dir, &copy_files);
            }

            if validate {
                continue;
            }

            #[rustfmt::skip]
            write_result_to_file(
                "result/web_result.csv",
                &Vec::from([
                    ("language", meta_data.language.as_str()),
                    ("mode", meta_data.mode.as_str()),
                    ("version", language_version.as_str()),
                    ("framework", meta_data.framework.as_str()),
                    ("framework_stdlib", meta_data.framework_stdlib.to_string().as_str()),
                    ("framework_website", meta_data.framework_website.as_str()),
                    ("framework_flavor", meta_data.framework_flavor.as_str()),
                    ("framework_version", framework_version.as_str()),
                    ("concurrency", concurrency.to_string().as_str()),
                    ("path", dir.replace("benchmark/web/", "").as_str()),
                ]),
                &Vec::from([
                    ("rps_median", result.additional_data.get("rps_median").unwrap().to_string().as_str()),
                    ("rps_p99", result.additional_data.get("rps_p99").unwrap().to_string().as_str()),
                    ("latency_median", result.additional_data.get("latency_median").unwrap().to_string().as_str()),
                    ("latency_p99", result.additional_data.get("latency_p99").unwrap().to_string().as_str()),
                    ("memory_median", result.memory_median.to_string().as_str()),
                    ("memory_p99", result.memory_p99.to_string().as_str()),
                    ("errors", result.additional_data.get("errors").unwrap().to_string().as_str()),
                ]),
                take_bigger_rps,
            )
            .expect("Failed to write result to file");
        }
    }
}

#[derive(Deserialize)]
struct PeriodicTableElement {
    name: String,
    number: u8,
    group: u8,
    shells: Vec<u8>,
}

fn load_data() -> HashMap<String, PeriodicTableElement> {
    let data: String = fs::read_to_string("src/benchmark/web/data/static/data.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(data.as_str()).unwrap();

    let mut elements: HashMap<String, PeriodicTableElement> = HashMap::new();

    for (key, value) in json.as_object().unwrap() {
        let element: PeriodicTableElement = serde_json::from_value(value.clone()).unwrap();
        elements.insert(key.to_string(), element);
    }

    elements
}

fn prepare_requests(
    benchmark_url: &str,
    data: &HashMap<String, PeriodicTableElement>,
) -> Vec<PreparedHttpRequest> {
    [
        data.iter()
            .map(|(k, v)| {
                let url = format!(
                    "{}/api/v1/periodic-table/element?symbol={}",
                    benchmark_url, k
                );
                let expected_response = HashMap::from([
                    (
                        "name".to_string(),
                        SerializedValue::StringValue(v.name.to_string()),
                    ),
                    (
                        "number".to_string(),
                        SerializedValue::IntValue(v.number as i32),
                    ),
                    (
                        "group".to_string(),
                        SerializedValue::IntValue(v.group as i32),
                    ),
                ]);

                PreparedHttpRequest {
                    url,
                    expected_response,
                }
            })
            .collect::<Vec<PreparedHttpRequest>>(),
        data.iter()
            .map(|(k, v)| {
                let url = format!(
                    "{}/api/v1/periodic-table/shells?symbol={}",
                    benchmark_url, k
                );
                let expected_response = HashMap::from([(
                    "shells".to_string(),
                    SerializedValue::IntListValue(
                        v.shells.iter().map(|v| *v as i32).collect::<Vec<i32>>(),
                    ),
                )]);

                PreparedHttpRequest {
                    url,
                    expected_response,
                }
            })
            .collect::<Vec<PreparedHttpRequest>>(),
    ]
    .concat()
}

/// Returns the URL the load test sends its requests to.
/// Requests to a published port via localhost go through docker-proxy, a userspace TCP relay
/// adding noise to each request. Connecting to the container IP bypasses it (and the NAT).
/// This requires the Docker network to be attached to the host, which is not the case
/// e.g. with Docker Desktop or rootless Docker. Otherwise, the published port is used.
fn get_benchmark_url() -> String {
    if let Some((ip, gateway)) = get_container_network(crate::CONTAINER_NAME) {
        // The network is attached to the host if the host reaches the container from the gateway.
        // Connecting a UDP socket only selects the route, no packet is sent.
        let source = UdpSocket::bind("0.0.0.0:0").and_then(|socket| {
            socket.connect((ip, 3000))?;
            socket.local_addr()
        });
        if source.is_ok_and(|source| source.ip() == gateway) {
            let address = SocketAddr::new(ip, 3000);
            println!(" -> Sending requests to the container IP {address}");
            return format!("http://{address}");
        }
    }

    println!(" -> Container IP not reachable, sending requests to the published port");
    "http://localhost:3000".to_string()
}

/// Resets the request counter of the web data source and returns its value before the reset.
fn reset_data_source_counter() -> i32 {
    reqwest::blocking::get("http://localhost:3001/reset")
        .and_then(|response| response.text())
        .ok()
        .and_then(|counter| counter.parse::<i32>().ok())
        .expect("Failed to reset the request counter of the web data source. Does the benchmark listen on port 80? This port is reserved for the data source")
}

fn response_validator(response: &PendingValidationResponse) -> Result<(), String> {
    let json: serde_json::Value = {
        match serde_json::from_str::<serde_json::Value>(&response.body) {
            Ok(json) => json,
            Err(_) => return Err(format!("Failed to parse JSON: {}", &response.body)),
        }
    };

    for (key, expected_value) in response.expected_body {
        let actual_value = json.get(key).ok_or(&format!(
            r#"Expected "{key}": {expected_value} but this key does not exist"#,
        ))?;
        match expected_value {
            SerializedValue::StringValue(v) => {
                if actual_value.as_str().ok_or(&format!(
                    r#"Expected "{key}": {expected_value} but got <{actual_value:?}>"#,
                ))? != v
                {
                    return Err(format!(
                        r#"Expected "{key}": {expected_value} but got <{actual_value:?}>"#,
                    ));
                }
            }
            SerializedValue::IntValue(v) => {
                if actual_value.as_i64().ok_or(&format!(
                    r#"Expected "{key}": {expected_value} but got <{actual_value:?}>"#,
                ))? != *v as i64
                {
                    return Err(format!(
                        r#"Expected "{key}": {expected_value} but got <{actual_value:?}>"#,
                    ));
                }
            }
            SerializedValue::IntListValue(v) => {
                let actual_list = actual_value.as_array().ok_or(&format!(
                    r#"Expected "{key}": {expected_value} but got <{actual_value:?}>"#,
                ))?;
                if actual_list.len() != v.len() {
                    return Err(format!(
                        r#"Expected "{key}": {expected_value} but got <{actual_value:?}>"#,
                    ));
                }
                for (i, actual_value) in actual_list.iter().enumerate() {
                    if actual_value.as_i64().ok_or(&format!(
                        r#"Expected "{key}": {expected_value} but got <{actual_value:?}>"#,
                    ))? != v[i] as i64
                    {
                        return Err(format!(
                            r#"Expected "{key}": {expected_value} but got <{actual_value:?}>"#,
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

fn take_bigger_rps<'a>(old_values: &'a [&'a str], new_values: &'a [&'a str]) -> &'a [&'a str] {
    if old_values[0].parse::<i32>().unwrap() > new_values[0].parse::<i32>().unwrap() {
        println!(
            " -> Keeping old values (rps_median: {} > {})",
            old_values[0], new_values[0]
        );
        old_values
    } else {
        new_values
    }
}
