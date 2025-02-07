use rama::http::matcher::HttpMatcher;
use rama::http::response::Json;
use rama::http::server::HttpServer;
use rama::http::service::web::extract::Query;
use rama::http::service::web::match_service;
use rama::http::StatusCode;
use rama::rt::Executor;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let client = Arc::new(Client::new());
    let state = State { client };

    let http_service = match_service! {
        HttpMatcher::get("/api/v1/periodic-table/element") => get_element,
        HttpMatcher::get("/api/v1/periodic-table/shells") => get_shells,
        _ => StatusCode::NOT_FOUND,
    };

    HttpServer::auto(Executor::default())
        .listen_with_state(state, "0.0.0.0:3000", http_service)
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct SymbolQuery {
    symbol: String,
}

#[derive(Debug, Clone)]
struct State {
    client: Arc<Client>,
}

type Context = rama::Context<State>;

async fn get_element(Query(query): Query<SymbolQuery>, ctx: Context) -> Json<ElementResponse> {
    let json: HashMap<String, DataSourceElement> = ctx
        .state()
        .client
        .get("http://web-data-source/element.json")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entry: &DataSourceElement = json.get(&query.symbol).unwrap();
    Json(ElementResponse {
        name: entry.name.clone(),
        number: entry.number,
        group: entry.group,
    })
}

async fn get_shells(Query(query): Query<SymbolQuery>, ctx: Context) -> Json<ShellsResponse> {
    let json: HashMap<String, Vec<u8>> = ctx
        .state()
        .client
        .get("http://web-data-source/shells.json")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
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
