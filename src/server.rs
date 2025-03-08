use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, LazyLock, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use rustls::ServerConfig as TlsServerConfig;
use rustls::ClientConfig as TlsClientConfig;
use serde::Deserialize;
use crate::{HttpHandler};
use crate::service::ExecutorService;
use crate::service::ServiceExecutor;

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
    pub request: Vec<HttpHandler>,
    pub response: Vec<HttpHandler>,
}

#[derive(Default)]
pub struct ServerConfig {
    pub worker_threads: usize,
    pub worker_thread_name: String,
    pub port: u16,
    pub config_dir: String,
    pub tls_enabled: bool,
    pub tls_server_config: Option<TlsServerConfig>,
    pub tls_client_config: Option<TlsClientConfig>,
    pub paths: Vec<PathConfig>,
}

pub struct ServerBuilder {
    worker_threads: usize,
    worker_thread_name: String,
    config_dir: String,
    port: u16,
    tls_enabled: bool,
    tls_server_config: Option<TlsServerConfig>,
    tls_client_config: Option<TlsClientConfig>,
    paths: Vec<PathConfig>,
}

#[allow(dead_code)]
impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            worker_threads: 1,
            worker_thread_name: "WT".to_string(),
            config_dir: "./config.json".to_string(),
            port: 8080,
            tls_enabled: false,
            tls_server_config: None,
            tls_client_config: None,
            paths: Vec::new(),
        }
    }

    pub fn worker_threads(&mut self, value: usize) -> &mut Self {
        self.worker_threads = value;
        self
    }

    pub fn worker_thread_name(&mut self, value: &str) -> &mut Self {
        self.worker_thread_name = value.to_string();
        self
    }

    pub fn port(&mut self, value: u16) -> &mut Self {
        self.port = value;
        self
    }

    pub fn tls_server_config(&mut self, value: TlsServerConfig) -> &mut Self {
        self.tls_enabled = true;
        self.tls_server_config = Some(value);
        self
    }

    pub fn tls_client_config(&mut self, value: TlsClientConfig) -> &mut Self {
        self.tls_enabled = true;
        self.tls_client_config = Some(value);
        self
    }

    pub fn add_path(&mut self, value: PathConfig) -> &mut Self {
        self.paths.push(value);
        self
    }

    pub fn build(self) -> ServerConfig {
        let mut config = ServerConfig::default();
        config.port = self.port;
        config.worker_thread_name = self.worker_thread_name;
        config.worker_threads = self.worker_threads;
        config.tls_enabled = self.tls_enabled;
        config.tls_server_config = self.tls_server_config;
        config.paths = self.paths;
        config
    }
}

pub fn run_server(config: ServerConfig) -> Result<(), ()> {
    let server_thread_name = config.worker_thread_name.clone();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.worker_threads)
        .thread_name_fn(move || {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("{}-{}", server_thread_name.as_str(), id)
        })
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async {
        let port = config.port;
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), port);
        let incoming = match TcpListener::bind(&addr).await {
            Ok(incoming) => incoming,
            Err(_) => todo!()
        };

        /* handle https server connections */
        if config.tls_enabled {
            println!("Starting to serve on https://{}", addr);
            if let Some(tls_config) = &config.tls_server_config {
                let tls_config = Arc::new(tls_config.clone());
                let tls_acceptor = TlsAcceptor::from(tls_config);
                let exec_svc = ExecutorService::new(Arc::new(config));
                loop {
                    let (tcp_stream, remote_addr) = match incoming.accept().await {
                        Ok(stream) => stream,
                        Err(_) => todo!()
                    };
                    let tls_acceptor = tls_acceptor.clone();
                    let mut exec_svc_clone = exec_svc.clone();
                    exec_svc_clone.set_src(remote_addr);
                    tokio::spawn(async move {
                        match tls_acceptor.accept(tcp_stream).await {
                            Ok(tls_stream) => {
                                let io = TokioIo::new(tls_stream);

                                if let Err(err) = auto::Builder::new(ServiceExecutor)
                                    .serve_connection(io, exec_svc_clone)
                                    .await {
                                    eprintln!("failed to serve connection: {:#}", err);
                                }
                            },
                            Err(err) => {
                                eprintln!("failed to perform tls handshake: {err:#?}");
                            }
                        };
                    });
                }
            } else {
                panic!("TLS misconfiguration!")
            }

        /* handle http server connections */
        } else {
            println!("Starting to serve on http://{}", addr);
            let exec_svc = ExecutorService::new(Arc::new(config));
            loop {
                let (tcp_stream, remote_addr) = match incoming.accept().await {
                    Ok(stream) => stream,
                    Err(_) => todo!()
                };
                let io = TokioIo::new(tcp_stream);
                let mut exec_svc_clone = exec_svc.clone();
                exec_svc_clone.set_src(remote_addr);
                tokio::spawn(async move {
                    if let Err(err) = auto::Builder::new(ServiceExecutor)
                        .serve_connection(io, exec_svc_clone)
                        .await {
                        eprintln!("failed to serve connection: {:#}", err);
                    }
                });

            }
        }
    })
}