use std::collections::HashMap;
use std::sync::Arc;
use axum::{routing::get, Json, Router};
use axum::extract::{Query, State};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());
    let app = Router::new()
        .route("/api/v1/periodic-table/element", get(get_element))
        .route("/api/v1/periodic-table/shells", get(get_shells))
        .with_state(client.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct SymbolQuery {
    symbol: String,
}

async fn get_element(
    State(client): State<Arc<Client>>,
    query: Query<SymbolQuery>,
) -> Json<ElementResponse> {
    let json: HashMap<String, DataSourceElement> = client.get("http://127.0.0.1:5002/element.json").send().await.unwrap().json().await.unwrap();
    let entry: &DataSourceElement = json.get(&query.symbol).unwrap();
    Json(ElementResponse {
        name: entry.name.clone(),
        number: entry.number,
        group: entry.group,
    })
}

async fn get_shells(
    State(client): State<Arc<Client>>,
    query: Query<SymbolQuery>,
) -> Json<ShellsResponse> {
    let json: HashMap<String, Vec<u8>> = client.get("http://127.0.0.1:5002/shells.json").send().await.unwrap().json().await.unwrap();
    Json(ShellsResponse {
        shells: json.get(&query.symbol).unwrap().clone(),
    })
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
