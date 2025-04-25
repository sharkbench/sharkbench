use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::Client;

#[derive(Deserialize)]
struct SymbolQuery {
    symbol: String,
}

async fn get_element(
    client: web::Data<Client>,
    query: web::Query<SymbolQuery>,
) -> impl Responder {
    let json: HashMap<String, DataSourceElement> = client.get("http://web-data-source/element.json").send().await.unwrap().json().await.unwrap();
    let entry: &DataSourceElement = json.get(&query.symbol).unwrap();
    HttpResponse::Ok().json(ElementResponse {
        name: entry.name.clone(),
        number: entry.number,
        group: entry.group,
    })
}

async fn get_shells(
    client: web::Data<Client>,
    query: web::Query<SymbolQuery>,
) -> impl Responder {
    let json: HashMap<String, Vec<u8>> = client.get("http://web-data-source/shells.json").send().await.unwrap().json().await.unwrap();
    HttpResponse::Ok().json(ShellsResponse {
        shells: json.get(&query.symbol).unwrap().clone(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = web::Data::new(Client::new());
    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .route("/api/v1/periodic-table/element", web::get().to(get_element))
            .route("/api/v1/periodic-table/shells", web::get().to(get_shells))
    })
        .bind("0.0.0.0:3000")?
        .run()
        .await
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
