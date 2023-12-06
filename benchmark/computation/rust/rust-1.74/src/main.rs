use axum::{extract::{Query}, response::Html, routing::get, Router};
use serde::{Deserialize};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct BenchmarkQuery {
    iterations: usize,
}

async fn handler(
    pagination: Query<BenchmarkQuery>,
) -> Html<String> {
    let pi = calc_pi(pagination.iterations);
    Html(pi.to_string())
}

fn calc_pi(iterations: usize) -> f64 {
    let mut pi = 0.0;
    let mut denominator = 1.0;
    for x in 0..iterations {
        if x % 2 == 0 {
            pi = pi + (1.0 / denominator);
        } else {
            pi = pi - (1.0 / denominator);
        }
        denominator = denominator + 2.0;
    }
    pi = pi * 4.0;
    pi
}
