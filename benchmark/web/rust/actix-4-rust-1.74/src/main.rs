use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

#[derive(Deserialize)]
struct SymbolQuery {
    symbol: String,
}

async fn fetch_data(url: &str, client: web::Data<Client>) -> Result<Value, reqwest::Error> {
    client.get(url).send().await?.json::<Value>().await
}

async fn get_element(
    client: web::Data<Client>,
    query: web::Query<SymbolQuery>,
) -> impl Responder {
    let json = fetch_data("http://web-data-source/data.json", client).await.unwrap();
    let entry = json.get(&query.symbol).unwrap();
    HttpResponse::Ok().json({
        let mut response = HashMap::new();
        response.insert("name", entry["name"].clone());
        response.insert("number", entry["number"].clone());
        response.insert("group", entry["group"].clone());
        response
    })
}

async fn get_shells(
    client: web::Data<Client>,
    query: web::Query<SymbolQuery>,
) -> impl Responder {
    let json = fetch_data("http://web-data-source/data.json", client).await.unwrap();
    let entry = json.get(&query.symbol).unwrap();
    HttpResponse::Ok().json({
        let mut response = HashMap::new();
        response.insert("shells", entry["shells"].clone());
        response
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
