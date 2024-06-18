use axum::Router;
use axum::routing::get;

#[tokio::main]
async fn main() {
    let element_file: &'static String = Box::leak(Box::new(std::fs::read_to_string("static/element.json").unwrap()));
    let shells_file: &'static String = Box::leak(Box::new(std::fs::read_to_string("static/shells.json").unwrap()));

    let app = Router::new()
        .route("/element.json", get(|| async { element_file.to_owned() }))
        .route("/shells.json", get(|| async { shells_file.to_owned() }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
