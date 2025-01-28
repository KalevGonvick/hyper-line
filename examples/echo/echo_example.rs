use std::future::Future;
use std::pin::Pin;
use hyper::Response;
use log::info;
use hyper_line::config::{HttpMethod, PathConfig};
use hyper_line::handler::Handler;
use hyper_line::handler::reverse_proxy_handler::{ProxyConfig, ReverseProxyHandler};
use hyper_line::server::ServerBuilder;

struct ExampleEchoHandler;
impl Handler for ExampleEchoHandler {
    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        context: &'i2 mut hyper_line::exchange::Exchange
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
            context.save_response(echoed_response);
            Ok(())
        })
    }
}

fn main() {
    hyper_line::logger::setup_logger();
    let mut builder = ServerBuilder::new();
    builder
        .worker_thread_name("WT")
        .worker_threads(1)
        .port(8081)
        .add_path(PathConfig {
            path: "/test".to_string(),
            method: HttpMethod::Post,
            request: vec![Box::new(ExampleEchoHandler{})],
            response: vec![],
        });

    if let Err(e) = hyper_line::server::run_server(builder.build()) {
        eprintln!("FAILED: {:?}", e);
        std::process::exit(1);
    }
}