pub mod reverse_proxy_handler;
pub mod exchange_trace_handler;
pub mod request_echo_handler;

use std::convert::Infallible;
use std::future::Future;
use std::net::{SocketAddr, SocketAddrV4};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use hyper::service::Service;
use crate::config::{HttpMethod, ServerConfig};
use crate::exchange::Exchange;


pub trait Handler: Send
{
    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        context: &'i2 mut Exchange
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o;
}



