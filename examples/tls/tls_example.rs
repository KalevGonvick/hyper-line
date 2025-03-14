use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use hyper::Response;
use log::info;
use hyper_line::server::{HttpMethod, PathConfig, ServerBuilder};
use hyper_line::handler::Handler;
use hyper_line::handler::reverse_proxy_handler::{ProxyConfig, ReverseProxyHandler};
use rustls::ServerConfig as TlsServerConfig;
use hyper_line::{cert_manager, HttpRequest, HttpResponse};
use hyper_line::exchange::Exchange;

struct ExampleEchoHandler;
impl Handler<HttpRequest, HttpResponse> for ExampleEchoHandler {
    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        context: &'i2 mut Exchange<HttpRequest, HttpResponse>
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o
    {
        Box::pin(async move {
            info!("Echo handler");
            let consumed = context.consume_request().unwrap();
            let (_, request) = consumed.into_parts();
            let echoed_response = Response::new(request);
            context.save_output(echoed_response);
            Ok(())
        })
    }
}

fn main() {
    hyper_line::logger::setup_logger();

    let private_key = cert_manager::load_private_key("./examples/tls/server.rsa").unwrap();
    let public_key = cert_manager::load_certs("./examples/tls/server.pem").unwrap();

    let tls_server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(public_key, private_key)
        .unwrap();

    let mut builder = ServerBuilder::new();
    builder
        .worker_thread_name("WT")
        .worker_threads(1)
        .port(8081)
        .tls_server_config(tls_server_config)
        .add_path(PathConfig {
            path: "/test".to_string(),
            method: HttpMethod::Post,
            request: vec![Arc::new(ExampleEchoHandler{})],
            response: vec![],
        });

    if let Err(e) = hyper_line::server::run_server(builder.build()) {
        eprintln!("FAILED: {:?}", e);
        std::process::exit(1);
    }
}