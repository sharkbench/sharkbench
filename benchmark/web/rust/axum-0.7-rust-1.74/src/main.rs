use std::collections::HashMap;
use std::sync::Arc;
use axum::{routing::get, Json, Router, Extension};
use axum::extract::Query;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());
    let app = Router::new()
        .route("/api/v1/periodic-table/element", get(get_element))
        .route("/api/v1/periodic-table/shells", get(get_shells))
        .layer(Extension(client));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn fetch_data(url: &str, client: Arc<Client>) -> reqwest::Result<HashMap<String, DataSourceElement>> {
    return client.get(url).send().await?.json::<HashMap<String, DataSourceElement>>().await;
}

#[derive(Deserialize)]
struct SymbolQuery {
    symbol: String,
}

async fn get_element(
    Extension(client): Extension<Arc<Client>>,
    query: Query<SymbolQuery>,
) -> Json<ElementResponse> {
    let json: HashMap<String, DataSourceElement> = fetch_data("http://web-data-source/data.json", client).await.unwrap();
    let entry: &DataSourceElement = json.get(&query.symbol).unwrap();
    Json(ElementResponse {
        name: entry.name.clone(),
        number: entry.number,
        group: entry.group,
    })
}

async fn get_shells(
    Extension(client): Extension<Arc<Client>>,
    query: Query<SymbolQuery>,
) -> Json<ShellsResponse> {
    let json: HashMap<String, DataSourceElement> = fetch_data("http://web-data-source/data.json", client).await.unwrap();
    let entry: &DataSourceElement = json.get(&query.symbol).unwrap();
    Json(ShellsResponse {
        shells: entry.shells.clone(),
    })
}

#[derive(Deserialize, Serialize)]
struct ElementResponse {
    name: String,
    number: u8,
    group: u8,
}

#[derive(Deserialize, Serialize)]
struct ShellsResponse {
    shells: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataSourceElement {
    name: String,
    number: u8,
    group: u8,
    shells: Vec<u8>,
}
