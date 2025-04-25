use rama::{
    dns::HickoryDns,
    error::{BoxError, ErrorContext, OpaqueError},
    http::{
        client::{HttpClientService, HttpConnector},
        layer::{
            decompression::DecompressionLayer, map_response_body::MapResponseBodyLayer,
            required_header::AddRequiredRequestHeadersLayer, timeout::TimeoutLayer,
            trace::TraceLayer,
        },
        response::Json,
        server::HttpServer,
        service::{
            client::HttpClientExt,
            web::{extract::Query, Router},
        },
        Body, BodyExtractExt, Request, Response, StatusCode,
    },
    layer::MapErrLayer,
    net::{
        address::{Authority, SocketAddress},
        client::{
            ConnStoreFiFoReuseLruDrop, ConnectorService, EstablishedClientConnection, Pool,
            PooledConnector, ReqToConnID,
        },
        http::RequestContext,
        Protocol,
    },
    rt::Executor,
    service::BoxService,
    tcp::client::service::TcpConnector,
    Layer, Service,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use std::{num::NonZeroU16, sync::Arc};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt, EnvFilter};

type Client = Arc<BoxService<(), Request, Response, OpaqueError>>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
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
    let http_service = TraceLayer::new_for_http().into_layer(app);

    let addr = SocketAddress::default_ipv4(3000);
    tracing::info!("run benchmark rama server @ {addr}");
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
    let conn_pool = Pool::<ConnStoreFiFoReuseLruDrop<HttpClientService<Body>, BasicConnID>>::new(
        NonZeroU16::new(128).context("create NonZeroU16(128)")?,
        NonZeroU16::new(1024).context("create NonZeroU16(1024)")?,
    )
    .context("create conn pool")?;

    let connector = PooledConnector::new(
        HttpConnector::new(TcpConnector::new().with_dns(try_new_dns_resolver()?)),
        conn_pool,
        BasicHttpConnId,
    );

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

#[derive(Debug)]
#[non_exhaustive]
struct BasicHttpConnId;
type BasicConnID = (Protocol, Authority);

impl<Body> ReqToConnID<(), Request<Body>> for BasicHttpConnId {
    type ConnID = BasicConnID;

    fn id(
        &self,
        ctx: &rama::Context<()>,
        req: &Request<Body>,
    ) -> Result<Self::ConnID, OpaqueError> {
        let req_ctx = match ctx.get::<RequestContext>() {
            Some(ctx) => ctx,
            None => &RequestContext::try_from((ctx, req))?,
        };

        Ok((req_ctx.protocol.clone(), req_ctx.authority.clone()))
    }
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
