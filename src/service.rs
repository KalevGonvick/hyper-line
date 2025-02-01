use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode};
use hyper::service::Service;
use rustls::ServerConfig as TlsServerConfig;
use crate::config::{HttpMethod, ServerConfig};
use crate::exchange::Exchange;
use crate::handler::Handler;

#[derive(Clone)]
pub struct ServiceExecutor;

impl<F> hyper::rt::Executor<F> for ServiceExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

#[derive(Clone)]
pub struct ExecutorService {
    config: Arc<ServerConfig>,
    src: Option<SocketAddr>,
}

impl ExecutorService {
    pub fn new(
        config: Arc<ServerConfig>
    ) -> Self {
        Self {
            config,
            src: None,
        }
    }

    pub fn set_src(&mut self, src: SocketAddr) {
        self.src = Some(src);
    }

    pub(self) async fn execute_handler_chain(&self, exchange: &mut Exchange, handlers: &Vec<Box<dyn Handler + Sync + Send + 'static>>) -> Result<(), ()> {
        for handler in handlers.iter() {
            match handler.process(exchange).await {
                Ok(_) => {},
                Err(_) => return Err(()),
            }
        }
        Ok(())
    }

    pub(self) fn create_error_response(status_code: StatusCode) -> Response<UnsyncBoxBody<Bytes, Infallible>> {
        let mut res = Response::new(UnsyncBoxBody::new(Empty::<Bytes>::new()));
        *res.status_mut() = status_code;
        res
    }

    pub(crate) fn ssl_config(&self) -> &Option<TlsServerConfig> {
        &self.config.tls_server_config
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
            let src = match exec_svc_context.src {
                Some(s) => s,
                None => panic!("Invalid source IP!")
            };
            let mut exchange = Exchange::new(src, exec_svc_context.config.clone());
            for path in &exec_svc_context.config.paths {
                let req_method = &req.method().as_str();
                let http_method = match HttpMethod::from_str(req_method) {
                    Ok(method) => method,
                    Err(_) => panic!("Could not convert method {}", &req.method().as_str())
                };
                if http_method == path.method && req.uri().path().starts_with(&path.path) {
                    exchange.buffer_request(req).await.unwrap();

                    /* execute request chain */
                    match exec_svc_context.execute_handler_chain(&mut exchange, &path.request).await {
                        Ok(_) => log::trace!("Request handlers completed successfully."),
                        Err(_) => return Ok(Self::create_error_response(StatusCode::INTERNAL_SERVER_ERROR))
                    };

                    /* execute response chain */
                    match exec_svc_context.execute_handler_chain(&mut exchange, &path.response).await {
                        Ok(_) => log::trace!("Response handlers completed successfully."),
                        Err(_) => return Ok(Self::create_error_response(StatusCode::INTERNAL_SERVER_ERROR))
                    }
                    return Ok(exchange.consume_response().unwrap())
                }
            }
            Ok(Self::create_error_response(StatusCode::NOT_FOUND))
        };

        Box::pin(async { fut.await })
    }
}