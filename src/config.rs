use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use serde::Deserialize;
use rustls::{ClientConfig as TlsClientConfig, ServerConfig as TlsServerConfig};
use crate::handler::Handler;

type ConfigError = Box<dyn std::error::Error>;

#[derive(Default)]
pub struct HandlerRegister {
    registered: HashMap<String, Box<dyn Handler>>
}

impl HandlerRegister
{
    pub fn register(&mut self, handler_name: String, _handler_instance: Box<dyn Handler>) -> Result<(), ConfigError> {
        if !self.registered.contains_key(&handler_name) {

        }
        Err(DuplicateHandlerError.into())
    }
}

#[derive(Debug, Clone)]
struct DuplicateHandlerError;

impl Display for DuplicateHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duplicate handler defined.")
    }
}
impl std::error::Error for DuplicateHandlerError {}

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
pub struct PathConfig {
    pub path: String,
    pub method: HttpMethod,
    pub request: Vec<Box<dyn Handler + Sync + Send + 'static>>,
    pub response: Vec<Box<dyn Handler + Sync + Send + 'static>>,
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