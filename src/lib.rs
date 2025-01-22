mod handler;
pub mod config;
mod service;
pub mod status;
pub mod callback;
pub mod attachment_key;
mod exchange;
mod cert_manager;
mod logger;
mod server;

use std::io::Write;
use std::net::{SocketAddr};
use std::sync::Arc;
use std::{env, fs, io, thread};
use std::any::{Any};
use std::convert::Infallible;
use std::future::Future;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};


use http_body_util::combinators::UnsyncBoxBody;
use hyper::{
    body::{
        Body,
        Bytes
    },
    service::{
        Service
    }
};
use hyper_util::rt::{TokioIo};
use hyper_util::server::conn::auto;
use rustls::ServerConfig as TlsServerConfig;
use tokio::net::TcpListener;
use tokio_rustls::{rustls, TlsAcceptor};
use crate::config::{HttpMethod, PathConfig, ServerConfig};
use tracing::{debug, info, span, Instrument as _, Level};
use crate::cert_manager::{load_certs, load_private_key};
use crate::handler::ExecutorService;
use crate::service::ServiceExecutor;

pub type ChannelBody = UnsyncBoxBody<Bytes, Infallible>;

fn main() {
    logger::setup_logger();
    if let Err(e) = run_server() {
        eprintln!("FAILED: {:?}", e);
        std::process::exit(1);
    }
}





