use std::collections::HashMap;
use std::convert::Infallible;
use http_body_util::Full;
use hyper::{Request, Response};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client: &Client = Box::leak(Box::new(Client::new()));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(
                    |request: Request<hyper::body::Incoming>| async move {
                        let path: &str = request.uri().path();

                        if request.method() != hyper::Method::GET {
                            return Ok::<Response<Full<Bytes>>, Infallible>(Response::builder().status(405).body(Full::new(Bytes::from_static(b"405 Method Not Allowed"))).unwrap());
                        }

                        let symbol: Option<String> = request.uri().query().map(|query| {
                            let mut symbol = None;
                            for q in query.split('&') {
                                let mut split = q.split('=');
                                let key = split.next().unwrap();
                                let value = split.next().unwrap();
                                if key == "symbol" {
                                    symbol = Some(value.to_owned());
                                }
                            }
                            symbol
                        }).flatten();

                        if symbol.is_none() {
                            return Ok::<Response<Full<Bytes>>, Infallible>(Response::builder().status(400).body(Full::new(Bytes::from_static(b"400 Bad Request"))).unwrap());
                        }

                        let symbol = symbol.unwrap();

                        Ok::<Response<Full<Bytes>>, Infallible>(Response::new(Full::new(Bytes::from(match path {
                            "/api/v1/periodic-table/element" => get_element(client, &symbol).await.into_bytes(),
                            "/api/v1/periodic-table/shells" => get_shells(client, &symbol).await.into_bytes(),
                            _ => b"404 Not Found".to_vec(),
                        }))))
                    }
                ))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn get_element(
    client: &Client,
    symbol: &String,
) -> String {
    let json: HashMap<String, DataSourceElement> = client.get("http://web-data-source/element.json").send().await.unwrap().json().await.unwrap();
    let entry: &DataSourceElement = json.get(&symbol as &String).unwrap();
    serde_json::to_string(&ElementResponse {
        name: entry.name.clone(),
        number: entry.number,
        group: entry.group,
    }).unwrap()
}

async fn get_shells(
    client: &Client,
    symbol: &String,
) -> String {
    let json: HashMap<String, Vec<u8>> = client.get("http://web-data-source/shells.json").send().await.unwrap().json().await.unwrap();
    serde_json::to_string(&ShellsResponse {
        shells: json.get(&symbol as &String).unwrap().clone(),
    }).unwrap()
}

#[derive(Serialize)]
struct ElementResponse {
    name: String,
    number: u8,
    group: u8,
}

#[derive(Serialize)]
struct ShellsResponse {
    shells: Vec<u8>,
}

#[derive(Deserialize, Debug)]
struct DataSourceElement {
    name: String,
    number: u8,
    group: u8,
}
