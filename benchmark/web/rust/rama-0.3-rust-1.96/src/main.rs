use rama::{
    Layer, Service,
    error::{BoxError, ErrorContext, ErrorExt, extra::OpaqueError},
    http::{
        Body, BodyExtractExt, Request, Response, StatusCode,
        body::OptionalBody,
        client::EasyHttpWebClient,
        layer::{
            decompression::DecompressionLayer, error_handling::ErrorHandlerLayer,
            map_response_body::MapResponseBodyLayer,
            required_header::AddRequiredRequestHeadersLayer, timeout::TimeoutLayer,
        },
        server::HttpServer,
        service::{
            client::HttpClientExt,
            web::{
                Router,
                extract::{Query, State as StateExtractor},
                response::{IntoResponse, Json},
            },
        },
    },
    layer::MapErrLayer,
    net::address::SocketAddress,
    rt::Executor,
    service::BoxService,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[cfg(feature = "tracing")]
use ::{
    rama::http::layer::trace::TraceLayer,
    rama::telemetry::tracing::{
        self,
        level_filters::LevelFilter,
        subscriber::{EnvFilter, fmt, layer::SubscriberExt as _, util::SubscriberInitExt},
    },
};

type Client = Arc<BoxService<Request, Response<OptionalBody<Body>>, OpaqueError>>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    #[cfg(feature = "tracing")]
    let _ = tracing::subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .try_init();

    let client = try_new_client()?;
    let state = State { client };

    let app = Router::new_with_state(state)
        .with_get("/api/v1/periodic-table/element", get_element)
        .with_get("/api/v1/periodic-table/shells", get_shells);

    let addr = SocketAddress::default_ipv4(3000);

    #[cfg(feature = "tracing")]
    let http_service = {
        tracing::info!("run benchmark rama server @ {addr}");
        Arc::new((TraceLayer::new_for_http(), ErrorHandlerLayer::new()).into_layer(app))
    };

    #[cfg(not(feature = "tracing"))]
    let http_service = Arc::new(ErrorHandlerLayer::new().into_layer(app));

    HttpServer::auto(Executor::default())
        .listen(addr, http_service)
        .await
}

#[derive(Deserialize)]
struct SymbolQuery {
    symbol: String,
}

#[derive(Debug, Clone)]
struct State {
    client: Client,
}

async fn get_element(
    Query(query): Query<SymbolQuery>,
    StateExtractor(state): StateExtractor<State>,
) -> Response {
    match try_fetch_json_data(state, "http://web-data-source/element.json").await {
        Ok(json) => {
            let json: HashMap<String, DataSourceElement> = json;
            let entry: &DataSourceElement = json.get(&query.symbol).unwrap();
            Json(ElementResponse {
                name: entry.name.clone(),
                number: entry.number,
                group: entry.group,
            })
            .into_response()
        }
        Err(err) => map_internal_error(err).into_response(),
    }
}

async fn get_shells(
    Query(query): Query<SymbolQuery>,
    StateExtractor(state): StateExtractor<State>,
) -> Response {
    match try_fetch_json_data(state, "http://web-data-source/shells.json").await {
        Ok(json) => {
            let json: HashMap<String, Vec<u8>> = json;
            Json(ShellsResponse {
                shells: json.get(&query.symbol).unwrap().clone(),
            })
            .into_response()
        }
        Err(err) => map_internal_error(err).into_response(),
    }
}

fn map_internal_error(err: OpaqueError) -> (StatusCode, String) {
    #[cfg(feature = "tracing")]
    tracing::error!(?err, "client fetch call failed (resp: status 500)");
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

async fn try_fetch_json_data<T: serde::de::DeserializeOwned + Send + 'static>(
    state: State,
    uri: &str,
) -> Result<T, OpaqueError> {
    state
        .client
        .get(uri)
        .send()
        .await
        .context("fetch json data")
        .into_opaque_error()?
        .try_into_json()
        .await
        .context("parse json data")
        .into_opaque_error()
}

fn try_new_client() -> Result<Client, OpaqueError> {
    let http_client = EasyHttpWebClient::connector_builder()
        .with_default_transport_connector()
        .with_default_dns_connector()
        .without_tls_proxy_support()
        .with_proxy_support()
        .without_tls_support()
        .with_default_http_connector(Executor::default())
        .try_with_default_connection_pool()
        .context("build pooled http connector")
        .into_opaque_error()?
        .build_client();

    Ok(Arc::new(
        (
            MapErrLayer::new(ErrorExt::into_opaque_error),
            TimeoutLayer::new(Duration::from_secs(10)),
            AddRequiredRequestHeadersLayer::new(),
            MapResponseBodyLayer::new(Body::new),
            DecompressionLayer::new(),
        )
            .into_layer(http_client)
            .boxed(),
    ))
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
