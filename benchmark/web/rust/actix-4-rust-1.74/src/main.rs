use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize)]
struct SymbolQuery {
    symbol: String,
}

async fn fetch_data(url: &str) -> Result<Value, reqwest::Error> {
    reqwest::get(url).await?.json::<Value>().await
}

async fn get_element(query: web::Query<SymbolQuery>) -> impl Responder {
    match fetch_data("http://web-data-source/data.json").await {
        Ok(json) => {
            if let Some(entry) = json.get(&query.symbol) {
                HttpResponse::Ok().json({
                    let mut response = HashMap::new();
                    response.insert("name", entry["name"].clone());
                    response.insert("number", entry["number"].clone());
                    response.insert("group", entry["group"].clone());
                    response
                })
            } else {
                HttpResponse::NotFound().finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_shells(query: web::Query<SymbolQuery>) -> impl Responder {
    match fetch_data("http://web-data-source/data.json").await {
        Ok(json) => {
            if let Some(entry) = json.get(&query.symbol) {
                HttpResponse::Ok().json({
                    let mut response = HashMap::new();
                    response.insert("shells", entry["shells"].clone());
                    response
                })
            } else {
                HttpResponse::NotFound().finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/v1/periodic-table/element", web::get().to(get_element))
            .route("/api/v1/periodic-table/shells", web::get().to(get_shells))
    })
        .bind("0.0.0.0:3000")?
        .run()
        .await
}
