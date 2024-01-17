#[macro_use] extern crate rocket;

use rocket::State;
use rocket::serde::{json::Json, Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

#[launch]
fn rocket() -> _ {
    let client = Arc::new(Client::new());
    rocket::build()
        .configure({
            let mut config = rocket::Config::release_default();
            config.address = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
            config.port = 3000;
            config
        })
        .mount("/api/v1/periodic-table", routes![get_element, get_shells])
        .manage(client)
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ElementResponse {
    name: String,
    number: u8,
    group: u8,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ShellsResponse {
    shells: Vec<u8>,
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct DataSourceElement {
    name: String,
    number: u8,
    group: u8,
}

#[get("/element?<symbol>")]
async fn get_element(client: &State<Arc<Client>>, symbol: &str) -> Json<ElementResponse> {
    let json: HashMap<String, DataSourceElement> = client.inner().get("http://web-data-source/element.json").send().await.unwrap().json().await.unwrap();
    let entry: &DataSourceElement = json.get(symbol).unwrap();
    Json(ElementResponse {
        name: entry.name.clone(),
        number: entry.number,
        group: entry.group,
    })
}

#[get("/shells?<symbol>")]
async fn get_shells(client: &State<Arc<Client>>, symbol: &str) -> Json<ShellsResponse> {
    let json: HashMap<String, Vec<u8>> = client.inner().get("http://web-data-source/shells.json").send().await.unwrap().json().await.unwrap();
    Json(ShellsResponse {
        shells: json.get(symbol).unwrap().clone(),
    })
}
