#![allow(dead_code)]
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

use std::convert::Infallible;


use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::Bytes;


use crate::service::ServiceExecutor;

pub type ChannelBody = UnsyncBoxBody<Bytes, Infallible>;

fn main() {
//    logger::setup_logger();
//    if let Err(e) = run_server() {
//        eprintln!("FAILED: {:?}", e);
//        std::process::exit(1);
//    }
}





