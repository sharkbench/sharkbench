use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::fs;
use std::sync::atomic::{AtomicU32, Ordering};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let element_file: &'static Bytes = Box::leak(Box::new(Bytes::from(fs::read_to_string("static/element.json").unwrap())));
    let shells_file: &'static Bytes = Box::leak(Box::new(Bytes::from(fs::read_to_string("static/shells.json").unwrap())));
    static COUNTER: AtomicU32 = AtomicU32::new(0);

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
                        .body(Full::new(match path {
                            "/element.json" => {
                                COUNTER.fetch_add(1, Ordering::Relaxed);
                                element_file.clone()
                            },
                            "/shells.json" => {
                                COUNTER.fetch_add(1, Ordering::Relaxed);
                                shells_file.clone()
                            },
                            "/reset" => {
                                let value = COUNTER.load(Ordering::SeqCst);
                                COUNTER.store(0, Ordering::SeqCst);
                                Bytes::from(value.to_string())
                            }
                            _ => Bytes::from("404 Not Found".as_bytes()),
                        }))
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
