use hyper_line::config::{HttpMethod, PathConfig};
use hyper_line::handler::reverse_proxy_handler::{ProxyConfig, ReverseProxyHandler};
use hyper_line::server::ServerBuilder;

fn main() {
    hyper_line::logger::setup_logger();
    let mut builder = ServerBuilder::new();
    builder
        .worker_thread_name("WT")
        .worker_threads(4)
        .port(8080)
        .add_path(PathConfig {
            path: "/test".to_string(),
            method: HttpMethod::Post,
            request: vec![Box::new(ReverseProxyHandler::new(ProxyConfig {
                destination_port: 8081,
                destination_host: "127.0.0.1".to_string(),
            }))],
            response: vec![],
        });

    if let Err(e) = hyper_line::server::run_server(builder.build()) {
        eprintln!("FAILED: {:?}", e);
        std::process::exit(1);
    }
}
