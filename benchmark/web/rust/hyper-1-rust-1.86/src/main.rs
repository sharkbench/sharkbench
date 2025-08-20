#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use smallstr::SmallString;
use std::collections::HashMap;
use std::convert::Infallible;
use std::rc::Rc;
use thiserror::Error;
use tokio::task::LocalSet;

type HttpClient = Client<HttpConnector, Full<Bytes>>;

#[derive(Debug, Error)]
enum AppError {
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Invalid URI: {0}")]
    Uri(#[from] hyper::http::uri::InvalidUri),

    #[error("HTTP request build error: {0}")]
    RequestBuild(#[from] hyper::http::Error),

    #[error("Body error: {0}")]
    Body(#[from] hyper_util::client::legacy::Error),

    #[error("Element not found")]
    ElementNotFound,

    #[error("Shells not found")]
    ShellsNotFound,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), AppError> {
    let local = LocalSet::new();

    local
        .run_until(async move {
            let client: Rc<HttpClient> =
                Rc::new(Client::builder(hyper_util::rt::TokioExecutor::new()).build_http());
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(_) => {
                        continue;
                    }
                };
                let io = TokioIo::new(stream);
                let client_ref = client.clone();

                tokio::task::spawn_local(async move {
                    http1::Builder::new()
                        .auto_date_header(false)
                        .pipeline_flush(true)
                        .serve_connection(
                            io,
                            service_fn(|request: Request<hyper::body::Incoming>| {
                                let client_ref = client_ref.clone();
                                async move { handle_request(client_ref, request).await }
                            }),
                        )
                        .await
                });
            }
        })
        .await;

    Ok(())
}

async fn handle_request(
    client: Rc<HttpClient>,
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path: &str = request.uri().path();

    if request.method() != hyper::Method::GET {
        return Ok(Response::builder()
            .status(405)
            .body(Full::new(Bytes::from_static(b"405 Method Not Allowed")))
            .unwrap());
    }

    let query = request.uri().query().unwrap_or("");
    for q in query.split('&') {
        let mut split = q.split('=');
        if let (Some(key), Some(value)) = (split.next(), split.next()) {
            if key == "symbol" {
                let response_body = match path {
                    "/api/v1/periodic-table/element" => match get_element(client, value).await {
                        Ok(data) => data,
                        Err(_) => {
                            return Ok(Response::builder()
                                .status(500)
                                .body(Full::new(Bytes::from_static(b"500 Internal Server Error")))
                                .unwrap());
                        }
                    },
                    "/api/v1/periodic-table/shells" => match get_shells(client, value).await {
                        Ok(data) => data,
                        Err(_) => {
                            return Ok(Response::builder()
                                .status(500)
                                .body(Full::new(Bytes::from_static(b"500 Internal Server Error")))
                                .unwrap());
                        }
                    },
                    _ => {
                        return Ok(Response::builder()
                            .status(404)
                            .body(Full::new(Bytes::from_static(b"404 Not Found")))
                            .unwrap());
                    }
                };
                return Ok(Response::new(Full::new(Bytes::from(response_body))));
            }
        }
    }

    Ok(Response::builder()
        .status(400)
        .body(Full::new(Bytes::from_static(b"400 Bad Request")))
        .unwrap())
}

async fn get_element(client: Rc<HttpClient>, symbol: &str) -> Result<Vec<u8>, AppError> {
    let uri: Uri = "http://web-data-source/element.json".parse()?;
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Full::new(Bytes::new()))?;

    let res = client.request(req).await?;
    let body_bytes = res.into_body().collect().await?.to_bytes();

    let json: HashMap<SmallString<[u8; 8]>, DataSourceElement> =
        serde_json::from_slice(&body_bytes)?;
    let entry = json.get(symbol).ok_or(AppError::ElementNotFound)?;

    let response = ElementResponse {
        name: entry.name.to_owned(),
        number: entry.number,
        group: entry.group,
    };

    Ok(serde_json::to_vec(&response)?)
}

async fn get_shells(client: Rc<HttpClient>, symbol: &str) -> Result<Vec<u8>, AppError> {
    let uri: Uri = "http://web-data-source/shells.json".parse()?;
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Full::new(Bytes::new()))?;

    let res = client.request(req).await?;
    let body_bytes = res.into_body().collect().await?.to_bytes();

    let json: HashMap<SmallString<[u8; 8]>, Vec<u8>> = serde_json::from_slice(&body_bytes)?;

    let shells = json.get(symbol).ok_or(AppError::ShellsNotFound)?;

    let response = ShellsResponse { shells: shells };

    Ok(serde_json::to_vec(&response)?)
}

#[derive(Serialize)]
struct ElementResponse {
    number: u8,
    group: u8,
    name: SmallString<[u8; 24]>,
}

#[derive(Serialize)]
struct ShellsResponse<'shell> {
    shells: &'shell Vec<u8>,
}

#[derive(Deserialize, Debug)]
struct DataSourceElement {
    number: u8,
    group: u8,
    name: SmallString<[u8; 24]>,
}
