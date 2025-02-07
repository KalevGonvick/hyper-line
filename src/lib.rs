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
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::Bytes;

pub type ChannelBody = UnsyncBoxBody<Bytes, Infallible>;






