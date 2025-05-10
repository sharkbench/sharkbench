use rama::{
    Layer, Service,
    dns::HickoryDns,
    error::{BoxError, ErrorContext, OpaqueError},
    http::{
        Body, BodyExtractExt, Request, Response, StatusCode,
        client::HttpConnector,
        layer::{
            decompression::DecompressionLayer, map_response_body::MapResponseBodyLayer,
            required_header::AddRequiredRequestHeadersLayer, timeout::TimeoutLayer,
        },
        server::HttpServer,
        service::{
            client::HttpClientExt,
            web::{Router, extract::Query, response::Json},
        },
    },
    layer::MapErrLayer,
    net::{
        address::SocketAddress,
        client::{
            ConnectorService, EstablishedClientConnection, pool::http::HttpPooledConnectorBuilder,
        },
    },
    rt::Executor,
    service::BoxService,
    tcp::client::service::TcpConnector,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[cfg(feature = "tracing")]
use ::{
    rama::http::layer::trace::TraceLayer,
    tracing::level_filters::LevelFilter,
    tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt as _, util::SubscriberInitExt},
};

type Client = Arc<BoxService<(), Request, Response, OpaqueError>>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    #[cfg(feature = "tracing")]
    let _ = tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .try_init();

    let client = try_new_client()?;
    let state = State { client };

    let app = Router::new()
        .get("/api/v1/periodic-table/element", get_element)
        .get("/api/v1/periodic-table/shells", get_shells);

    let addr = SocketAddress::default_ipv4(3000);

    #[cfg(feature = "tracing")]
    let http_service = {
        tracing::info!("run benchmark rama server @ {addr}");
        TraceLayer::new_for_http().into_layer(app)
    };

    #[cfg(not(feature = "tracing"))]
    let http_service = app;

    HttpServer::auto(Executor::default())
        .listen_with_state(state, addr, http_service)
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

type Context = rama::Context<State>;

async fn get_element(
    Query(query): Query<SymbolQuery>,
    ctx: Context,
) -> Result<Json<ElementResponse>, (StatusCode, String)> {
    let json: HashMap<String, DataSourceElement> =
        try_fetch_json_data(ctx, "http://web-data-source/element.json")
            .await
            .map_err(map_internal_error)?;
    let entry: &DataSourceElement = json.get(&query.symbol).unwrap();
    Ok(Json(ElementResponse {
        name: entry.name.clone(),
        number: entry.number,
        group: entry.group,
    }))
}

async fn get_shells(
    Query(query): Query<SymbolQuery>,
    ctx: Context,
) -> Result<Json<ShellsResponse>, (StatusCode, String)> {
    let json: HashMap<String, Vec<u8>> =
        try_fetch_json_data(ctx, "http://web-data-source/shells.json")
            .await
            .map_err(map_internal_error)?;
    Ok(Json(ShellsResponse {
        shells: json.get(&query.symbol).unwrap().clone(),
    }))
}

fn map_internal_error(err: OpaqueError) -> (StatusCode, String) {
    #[cfg(feature = "tracing")]
    tracing::error!(?err, "client fetch call failed (resp: status 500)");
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

async fn try_fetch_json_data<T: serde::de::DeserializeOwned + Send + 'static>(
    ctx: Context,
    uri: &str,
) -> Result<T, OpaqueError> {
    ctx.state()
        .client
        .get(uri)
        .send(Default::default())
        .await
        .context("fetch json data")?
        .try_into_json()
        .await
        .context("parse json data")
}

fn try_new_client() -> Result<Client, OpaqueError> {
    let connector = HttpPooledConnectorBuilder::new()
        .max_active(128)
        .max_total(1024)
        .build(HttpConnector::new(
            TcpConnector::new().with_dns(try_new_dns_resolver()?),
        ))
        .context("build pooled http connector")?;

    let http_client = HttpClient { connector };

    Ok(Arc::new(
        (
            MapErrLayer::new(OpaqueError::from_boxed),
            TimeoutLayer::new(Duration::from_secs(10)),
            AddRequiredRequestHeadersLayer::new(),
            MapResponseBodyLayer::new(Body::new),
            DecompressionLayer::new(),
        )
            .into_layer(http_client)
            .boxed(),
    ))
}

#[derive(Debug)]
struct HttpClient<C> {
    connector: C,
}

impl<C> Service<(), Request> for HttpClient<C>
where
    C: ConnectorService<
            (),
            Request,
            Connection: Service<(), Request, Response = Response, Error: Into<BoxError>>,
            Error: Into<BoxError>,
        >,
{
    type Response = Response;
    type Error = OpaqueError;

    async fn serve(
        &self,
        ctx: rama::Context<()>,
        req: Request,
    ) -> Result<Self::Response, Self::Error> {
        let EstablishedClientConnection { ctx, req, conn } =
            self.connector.connect(ctx, req).await.map_err(Into::into)?;
        conn.serve(ctx, req)
            .await
            .map_err(|err| OpaqueError::from_boxed(err.into()))
            .context("client: serve http request")
    }
}

#[cfg(any(unix, target_os = "windows"))]
fn try_new_dns_resolver() -> Result<HickoryDns, OpaqueError> {
    HickoryDns::try_new_system()
}

#[cfg(not(any(unix, target_os = "windows")))]
fn try_new_dns_resolver() -> Result<ResolverConfig, OpaqueError> {
    Ok(HickoryDns::new_cloudflare())
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
