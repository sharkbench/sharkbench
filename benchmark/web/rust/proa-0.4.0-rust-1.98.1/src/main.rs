use std::borrow::Cow;
use std::fmt;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode, Uri};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use bytes::Bytes;
use http::Request;
use http_body_util::{BodyExt as _, Empty};
use hyper::body::Incoming;
use hyper::client::conn::http1::{self, SendRequest};
use hyper_util::rt::TokioIo;
use proa::axum::{
    serve, FrameworkBuilder, LocalStreamExecutorConfig, DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT,
};
use serde::de::{DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use tokio::net::{TcpListener, TcpStream};

const UPSTREAM_ADDR: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5002));
const MAX_IDLE_CONNS: usize = 64;

static HOST_HEADER: HeaderValue = HeaderValue::from_static("127.0.0.1:5002");
static APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");

#[derive(Clone)]
struct UpstreamPool {
    idle: Arc<Mutex<Vec<SendRequest<Empty<Bytes>>>>>,
}

impl UpstreamPool {
    fn new() -> Self {
        Self {
            idle: Arc::new(Mutex::new(Vec::with_capacity(MAX_IDLE_CONNS))),
        }
    }

    fn pop_idle(&self) -> Option<SendRequest<Empty<Bytes>>> {
        let mut guard = self.idle.lock().unwrap();
        while let Some(sender) = guard.pop() {
            if !sender.is_closed() {
                return Some(sender);
            }
        }
        None
    }

    fn push_idle(&self, sender: SendRequest<Empty<Bytes>>) {
        if sender.is_closed() {
            return;
        }
        let mut guard = self.idle.lock().unwrap();
        if guard.len() < MAX_IDLE_CONNS {
            guard.push(sender);
        }
    }

    async fn connect(&self) -> Result<SendRequest<Empty<Bytes>>, ()> {
        let stream = TcpStream::connect(UPSTREAM_ADDR).await.map_err(|_| ())?;
        let _ = stream.set_nodelay(true);
        let (sender, conn) = http1::handshake(TokioIo::new(stream))
            .await
            .map_err(|_| ())?;
        tokio::spawn(async move {
            let _ = conn.await;
        });
        Ok(sender)
    }

    async fn fetch(&self, path: &'static str) -> Result<Bytes, ()> {
        if let Some(mut sender) = self.pop_idle() {
            if sender.ready().await.is_ok() {
                if let Ok(res) = sender.send_request(build_get_request(path)).await {
                    if let Ok(body) = read_body(res.into_body()).await {
                        self.push_idle(sender);
                        return Ok(body);
                    }
                }
            }
        }

        let mut sender = self.connect().await?;
        sender.ready().await.map_err(|_| ())?;
        let res = sender
            .send_request(build_get_request(path))
            .await
            .map_err(|_| ())?;
        let body = read_body(res.into_body()).await.map_err(|_| ())?;
        self.push_idle(sender);
        Ok(body)
    }
}

#[inline]
fn build_get_request(path: &'static str) -> Request<Empty<Bytes>> {
    let mut req = Request::new(Empty::new());
    *req.uri_mut() = Uri::from_static(path);
    req.headers_mut().insert(header::HOST, HOST_HEADER.clone());
    req
}

#[inline]
async fn read_body(mut body: Incoming) -> Result<Bytes, hyper::Error> {
    let mut first: Option<Bytes> = None;
    let mut buf: Vec<u8> = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame?;
        if let Ok(data) = frame.into_data() {
            if data.is_empty() {
                continue;
            }
            if buf.is_empty() {
                if let Some(prev) = first.take() {
                    buf.reserve(prev.len() + data.len());
                    buf.extend_from_slice(&prev);
                    buf.extend_from_slice(&data);
                } else {
                    first = Some(data);
                }
            } else {
                buf.extend_from_slice(&data);
            }
        }
    }
    Ok(match first {
        Some(bytes) => bytes,
        None => Bytes::from(buf),
    })
}

#[inline]
fn extract_symbol(uri: &Uri) -> Option<&str> {
    let query = uri.query()?;
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("symbol=") {
            return Some(value);
        }
    }
    None
}

#[inline]
fn json_response(body: Bytes) -> Response {
    let mut res = Response::new(Body::from(body));
    res.headers_mut()
        .insert(header::CONTENT_TYPE, APPLICATION_JSON.clone());
    res
}

#[inline]
fn status_response(status: StatusCode, body: &'static [u8]) -> Response {
    let mut res = Response::new(Body::from(Bytes::from_static(body)));
    *res.status_mut() = status;
    res
}

#[derive(Deserialize)]
struct BorrowedStr<'a>(#[serde(borrow)] Cow<'a, str>);

#[derive(Deserialize)]
struct BorrowedElement<'a> {
    #[serde(borrow)]
    name: Cow<'a, str>,
    number: u8,
    group: u8,
}

struct ElementLookup<'a> {
    target: &'a str,
}

impl<'de> DeserializeSeed<'de> for ElementLookup<'_> {
    type Value = Option<BorrowedElement<'de>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for ElementLookup<'_> {
    type Value = Option<BorrowedElement<'de>>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a map of periodic table elements")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut found: Option<BorrowedElement<'de>> = None;
        while let Some(key) = map.next_key::<BorrowedStr<'de>>()? {
            if found.is_none() && key.0.as_ref() == self.target {
                found = Some(map.next_value::<BorrowedElement<'de>>()?);
            } else {
                let _ = map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(found)
    }
}

struct ShellsStack {
    items: [u8; 16],
    len: usize,
}

impl<'de> Deserialize<'de> for ShellsStack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ShellsVisitor;

        impl<'de> Visitor<'de> for ShellsVisitor {
            type Value = ShellsStack;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an array of electron shell counts")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut items = [0u8; 16];
                let mut len = 0usize;
                while let Some(v) = seq.next_element::<u8>()? {
                    if len >= items.len() {
                        return Err(serde::de::Error::custom("too many shells"));
                    }
                    items[len] = v;
                    len += 1;
                }
                Ok(ShellsStack { items, len })
            }
        }

        deserializer.deserialize_seq(ShellsVisitor)
    }
}

struct ShellsLookup<'a> {
    target: &'a str,
}

impl<'de> DeserializeSeed<'de> for ShellsLookup<'_> {
    type Value = Option<ShellsStack>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for ShellsLookup<'_> {
    type Value = Option<ShellsStack>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a map of periodic table shells")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut found: Option<ShellsStack> = None;
        while let Some(key) = map.next_key::<BorrowedStr<'de>>()? {
            if found.is_none() && key.0.as_ref() == self.target {
                found = Some(map.next_value::<ShellsStack>()?);
            } else {
                let _ = map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(found)
    }
}

#[inline]
fn encode_element_json(entry: &BorrowedElement<'_>) -> Bytes {
    // `JsonBuffer::push_escaped` escapes each non-ASCII byte separately, which
    // would corrupt multi-byte UTF-8. Use serde_json for anything non-ASCII.
    if entry.name.len() > 48 || !entry.name.is_ascii() {
        #[derive(serde::Serialize)]
        struct Fallback<'a> {
            name: &'a str,
            number: u8,
            group: u8,
        }
        let encoded = serde_json::to_vec(&Fallback {
            name: entry.name.as_ref(),
            number: entry.number,
            group: entry.group,
        })
        .unwrap();
        return Bytes::from(encoded);
    }

    let mut buf = const_json::JsonBuffer::<160>::new();
    buf.push_lit("{\"name\":");
    const_json::ser::write_string(&mut buf, entry.name.as_ref());
    buf.push_lit(",\"number\":");
    const_json::ser::write_unsigned(&mut buf, entry.number as u128);
    buf.push_lit(",\"group\":");
    const_json::ser::write_unsigned(&mut buf, entry.group as u128);
    buf.push_byte(b'}');
    Bytes::copy_from_slice(buf.as_bytes())
}

#[inline]
fn encode_shells_json(shells: &ShellsStack) -> Bytes {
    let mut buf = const_json::JsonBuffer::<96>::new();
    buf.push_lit("{\"shells\":[");
    for (i, &count) in shells.items[..shells.len].iter().enumerate() {
        if i > 0 {
            buf.push_byte(b',');
        }
        const_json::ser::write_unsigned(&mut buf, count as u128);
    }
    buf.push_lit("]}");
    Bytes::copy_from_slice(buf.as_bytes())
}

async fn get_element(State(upstream): State<UpstreamPool>, uri: Uri) -> Response {
    let Some(symbol) = extract_symbol(&uri) else {
        return status_response(StatusCode::BAD_REQUEST, b"400 Bad Request");
    };
    let Ok(body) = upstream.fetch("/element.json").await else {
        return status_response(StatusCode::INTERNAL_SERVER_ERROR, b"500 Internal Server Error");
    };

    let mut de = serde_json::Deserializer::from_slice(&body);
    let entry = match (ElementLookup { target: symbol }).deserialize(&mut de) {
        Ok(entry) => entry,
        Err(_) => {
            return status_response(StatusCode::INTERNAL_SERVER_ERROR, b"500 Internal Server Error");
        }
    };
    if de.end().is_err() {
        return status_response(StatusCode::INTERNAL_SERVER_ERROR, b"500 Internal Server Error");
    }

    match entry {
        Some(entry) => json_response(encode_element_json(&entry)),
        None => status_response(StatusCode::NOT_FOUND, b"404 Not Found"),
    }
}

async fn get_shells(State(upstream): State<UpstreamPool>, uri: Uri) -> Response {
    let Some(symbol) = extract_symbol(&uri) else {
        return status_response(StatusCode::BAD_REQUEST, b"400 Bad Request");
    };
    let Ok(body) = upstream.fetch("/shells.json").await else {
        return status_response(StatusCode::INTERNAL_SERVER_ERROR, b"500 Internal Server Error");
    };

    let mut de = serde_json::Deserializer::from_slice(&body);
    let shells = match (ShellsLookup { target: symbol }).deserialize(&mut de) {
        Ok(shells) => shells,
        Err(_) => {
            return status_response(StatusCode::INTERNAL_SERVER_ERROR, b"500 Internal Server Error");
        }
    };
    if de.end().is_err() {
        return status_response(StatusCode::INTERNAL_SERVER_ERROR, b"500 Internal Server Error");
    }

    match shells {
        Some(shells) => json_response(encode_shells_json(&shells)),
        None => status_response(StatusCode::NOT_FOUND, b"404 Not Found"),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let pool = UpstreamPool::new();
    let routes = Router::new()
        .route("/api/v1/periodic-table/element", get(get_element))
        .route("/api/v1/periodic-table/shells", get(get_shells))
        .with_state(pool);

    let app = FrameworkBuilder::new()
        .with_dynamic_routes(routes)
        .without_rsjs_assets()
        .with_cross_origin_protection(false)
        .without_request_timeout()
        .with_local_stream_executor(LocalStreamExecutorConfig {
            worker_threads: 0,
            ..LocalStreamExecutorConfig::default()
        })
        .build();

    let listener = TcpListener::bind("0.0.0.0:5001").await.unwrap();
    serve(listener, app, DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT)
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_symbol_query_param() {
        let uri: Uri = "/api/v1/periodic-table/element?symbol=He".parse().unwrap();
        assert_eq!(extract_symbol(&uri), Some("He"));

        let multi: Uri = "/api/v1/periodic-table/shells?foo=1&symbol=Uue&bar=2"
            .parse()
            .unwrap();
        assert_eq!(extract_symbol(&multi), Some("Uue"));

        let missing: Uri = "/api/v1/periodic-table/element".parse().unwrap();
        assert_eq!(extract_symbol(&missing), None);
    }

    #[test]
    fn parses_and_encodes_element_without_hashmap() {
        let payload = br#"{"H":{"name":"Hydrogen","number":1,"group":1},"He":{"name":"Helium","number":2,"group":18}}"#;
        let mut de = serde_json::Deserializer::from_slice(payload);
        let entry = ElementLookup { target: "He" }
            .deserialize(&mut de)
            .unwrap()
            .unwrap();
        de.end().unwrap();

        let encoded = encode_element_json(&entry);
        assert_eq!(
            encoded.as_ref(),
            br#"{"name":"Helium","number":2,"group":18}"#
        );
    }

    #[test]
    fn parses_and_encodes_shells_without_hashmap() {
        let payload = br#"{"H":[1],"He":[2],"Uue":[2,8,18,32,32,18,8,1]}"#;
        let mut de = serde_json::Deserializer::from_slice(payload);
        let shells = ShellsLookup { target: "Uue" }
            .deserialize(&mut de)
            .unwrap()
            .unwrap();
        de.end().unwrap();

        let encoded = encode_shells_json(&shells);
        assert_eq!(
            encoded.as_ref(),
            br#"{"shells":[2,8,18,32,32,18,8,1]}"#
        );
    }
}
