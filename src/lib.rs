#![allow(dead_code)]
pub mod handler;
pub mod config;
mod service;
pub mod status;
pub mod callback;
pub mod attachment_key;
pub mod exchange;
pub mod cert_manager;
pub mod logger;
pub mod server;
use std::convert::Infallible;
use std::sync::Arc;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::Bytes;
use crate::handler::Handler;

pub type HttpBody = UnsyncBoxBody<Bytes, Infallible>;
pub type HttpRequest = http::Request<HttpBody>;
pub type HttpResponse = http::Response<HttpBody>;
pub type HttpHandler = Arc<dyn Handler<HttpRequest, HttpResponse> + Sync + Send + 'static>;






