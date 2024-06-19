use std::convert::Infallible;
use std::fs;
use http_body_util::Full;
use hyper::{Request, Response};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let element_file: &'static String = Box::leak(Box::new(fs::read_to_string("static/element.json").unwrap()));
    let shells_file: &'static String = Box::leak(Box::new(fs::read_to_string("static/shells.json").unwrap()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|request: Request<hyper::body::Incoming>| async move {
                    let path: &str = request.uri().path();

                    let response = Response::builder()
                        .header("content-type", "application/json")
                        .body(Full::new(Bytes::from(match path {
                            "/element.json" => element_file.as_bytes(),
                            "/shells.json" => shells_file.as_bytes(),
                            _ => b"404 Not Found",
                        })))
                        .unwrap();

                    Ok::<Response<Full<Bytes>>, Infallible>(response)
                }))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
