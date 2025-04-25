use rama::error::OpaqueError;
use rama::http::client::EasyHttpWebClient;
use rama::http::response::Json;
use rama::http::server::HttpServer;
use rama::http::service::client::HttpClientExt;
use rama::http::service::web::extract::Query;
use rama::http::service::web::Router;
use rama::http::{BodyExtractExt, Request, Response};
use rama::net::address::SocketAddress;
use rama::rt::Executor;
use rama::service::BoxService;
use rama::Service;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

type Client = BoxService<(), Request, Response, OpaqueError>;

#[tokio::main]
async fn main() {
    let client = Arc::new(EasyHttpWebClient::default().boxed());
    let state = State { client };

    let http_service = Router::new()
        .get("/api/v1/periodic-table/element", get_element)
        .get("/api/v1/periodic-table/shells", get_shells);

    HttpServer::auto(Executor::default())
        .listen_with_state(state, SocketAddress::local_ipv4(3000), http_service)
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
        .send(Default::default())
        .await
        .unwrap()
        .try_into_json()
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
        .send(Default::default())
        .await
        .unwrap()
        .try_into_json()
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
