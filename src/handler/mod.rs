pub mod reverse_proxy_handler;
pub mod exchange_trace_handler;
pub mod request_echo_handler;

use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
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

#[derive(Clone)]
pub struct ExecutorService {
    config: Arc<ServerConfig>,
    src: SocketAddr,
}

impl ExecutorService {
    pub fn new(
        config: Arc<ServerConfig>
    ) -> Self {
        Self {
            config,
            src: SocketAddr::V4("127.0.0.1".parse().unwrap()),
        }
    }

    pub fn set_src(&mut self, src: SocketAddr) {
        self.src = src;
    }
}

impl Service<Request<Incoming>> for ExecutorService
{
    type Response = Response<UnsyncBoxBody<Bytes, Infallible>>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(
        &self, req: Request<Incoming>
    ) -> Self::Future
    {
        let exec_svc_context = self.clone();
        let fut = async move {
            let _exchange = Exchange::new(exec_svc_context.src.clone());
            for path in &exec_svc_context.config.paths {
                let http_method = match HttpMethod::from_str(&req.method().as_str()) {
                    Ok(method) => method,
                    Err(_) => panic!("Could not convert method {}", &req.method().as_str())
                };
                if http_method == path.method && req.uri().path().starts_with(&path.path) {
                    //exchange.poll_read_request(req).await.unwrap();

//                    for middleware in &path.request.loaded_middleware {
//                        match match middleware.get() {
//                            Ok(x) => x,
//                            Err(_) => todo!(),
//                        }.process(&mut exchange).await {
//                            Ok(_) => {}
//                            Err(_) => todo!()
//                        };
//                    }
//                    let request_handler = &path.request.loaded_handler;
//                    match request_handler.get() {
//                        Ok(handler) => {
//                            println!("executing handler");
//                            match handler.process(&mut exchange).await {
//                                Ok(res) => res,
//                                Err(_) => todo!()
//                            }
//                        }
//                        Err(_) => todo!()
//                    };
//                    for middleware in &path.response.loaded_middleware {
//                        match match middleware.get() {
//                            Ok(x) => x,
//                            Err(_) => todo!(),
//                        }.process(&mut context).await {
//                            Ok(_) => {},
//                            Err(_) => todo!()
//                        }
//                    }
                    //return Ok(context.consume_response().1)
                    let mut default_response = Response::new(UnsyncBoxBody::new(Empty::<Bytes>::new()));
                    *default_response.status_mut() = hyper::StatusCode::NOT_FOUND;
                    return Ok(default_response);
                }
            }

            let mut default_response = Response::new(UnsyncBoxBody::new(Empty::<Bytes>::new()));
            *default_response.status_mut() = hyper::StatusCode::NOT_FOUND;
            Ok(default_response)
        };

        Box::pin(async { fut.await })
    }
}

