use axum::Router;
use memory_serve::{CacheControl, load_assets, MemoryServe};

const PORT: u16 = 80;

#[tokio::main]
async fn main() {
    let memory_router = MemoryServe::new(load_assets!("static"))
        .cache_control(CacheControl::Custom("no-cache, no-store, must-revalidate"))
        .html_cache_control(CacheControl::Custom("no-cache, no-store, must-revalidate"))
        .enable_gzip(false)
        .enable_brotli(false)
        .into_router();

    let app = Router::new().merge(memory_router);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT)).await.unwrap();
    println!("Server listening on port {}", PORT);
    axum::serve(listener, app).await.unwrap();
}
