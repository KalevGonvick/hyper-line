use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::Bytes;
use hyper::{Request, Response};
use serde::Deserialize;
use rustls::{ClientConfig as TlsClientConfig, ServerConfig as TlsServerConfig};
use crate::handler::Handler;

type ConfigError = Box<dyn std::error::Error>;

#[derive(Deserialize, Debug, Clone, PartialOrd, PartialEq, Default)]
pub enum HttpMethod {

    #[serde(alias = "OPTIONS")]
    Options,

    #[serde(alias = "GET")]
    #[default]
    Get,

    #[serde(alias = "POST")]
    Post,

    #[serde(alias = "PUT")]
    Put,

    #[serde(alias = "DELETE")]
    Delete,

    #[serde(alias = "HEAD")]
    Head,

    #[serde(alias = "TRACE")]
    Trace,

    #[serde(alias = "CONNECT")]
    Connect,

    #[serde(alias = "PATCH")]
    Patch
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "options" => Ok(HttpMethod::Options),
            "get" => Ok(HttpMethod::Get),
            "post" => Ok(HttpMethod::Post),
            "put" => Ok(HttpMethod::Put),
            "delete" => Ok(HttpMethod::Delete),
            "head" => Ok(HttpMethod::Head),
            "connect" => Ok(HttpMethod::Connect),
            "patch" => Ok(HttpMethod::Patch),
            _ => Err(())
        }
    }
}

#[derive(Default)]
pub struct PathConfig
{
    pub path: String,
    pub method: HttpMethod,
    pub request: Vec<Box<dyn Handler<Request<UnsyncBoxBody<Bytes, Infallible>>, Response<UnsyncBoxBody<Bytes, Infallible>>> + Sync + Send + 'static>>,
    pub response: Vec<Box<dyn Handler<Request<UnsyncBoxBody<Bytes, Infallible>>, Response<UnsyncBoxBody<Bytes, Infallible>>> + Sync + Send + 'static>>,
}

#[derive(Default)]
pub struct ServerConfig {
    pub(crate) worker_threads: usize,
    pub(crate)worker_thread_name: String,
    pub(crate) port: u16,
    pub(crate) config_dir: String,
    pub(crate) tls_enabled: bool,
    pub(crate) tls_server_config: Option<TlsServerConfig>,
    pub(crate) tls_client_config: Option<TlsClientConfig>,
    pub(crate) paths: Vec<PathConfig>,
}