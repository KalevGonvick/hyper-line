use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use rustls::ServerConfig as TlsServerConfig;
use crate::cert_manager::{load_certs, load_private_key};
use crate::config::{HttpMethod, PathConfig, ServerConfig};
use crate::handler::ExecutorService;
use crate::service::ServiceExecutor;

#[derive(Default)]
pub struct Server {
    config: ServerConfig
}

impl Server {
    pub fn new(
        config: ServerConfig
    ) -> Self {
        Self { config }
    }

    fn run_server() -> Result<(), ()> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name_fn(|| {
                static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
                let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
                format!("{}-{}", "WT", id)
            })
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        rt.block_on(async {
            let port = match env::args().nth(1) {
                Some(ref p) => match p.parse() {
                    Ok(d) => d,
                    Err(_) => todo!()
                },
                None => 8080,
            };

            let config_dir = match env::args().nth(2) {
                Some(ref d) => match d.parse() {
                    Ok(d) => d,
                    Err(_) => todo!()
                },
                None => String::from("./config/config.json")
            };

            //        let server_service_config = match config::load_from_file(config_dir) {
            //            None => todo!(),
            //            Some(config) => config
            //        };

            let mut server_service_config = ServerConfig::default();
            let mut path_config = PathConfig::default();
            path_config.method = HttpMethod::Post;
            path_config.path = "/testEndpoint".to_string();
            //        path_config.request.push()
            server_service_config.paths.push(path_config);

            let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), port);
            let server_certs = match load_certs("./server.pem") {
                Ok(certs) => certs,
                Err(_) => todo!()
            };
            let key = match load_private_key("./server.rsa") {
                Ok(key) => key,
                Err(_) => todo!()
            };

            println!("Starting to serve on https://{}", addr);

            let incoming = match TcpListener::bind(&addr).await {
                Ok(incoming) => incoming,
                Err(_) => todo!()
            };
            let mut server_config = match TlsServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(server_certs, key)
                .map_err(|_| todo!()) {
                Ok(config) => config,
                Err(_) => todo!()
            };

            server_config.alpn_protocols = vec![
                b"h2".to_vec(),
                b"http/1.1".to_vec()
            ];
            let server_tls_config = Arc::new(server_config);
            let tls_acceptor = TlsAcceptor::from(server_tls_config);

            let exec_svc = ExecutorService::new(Arc::new(server_service_config));

            loop {
                let (tcp_stream, remote_addr) = match incoming.accept().await {
                    Ok(stream) => stream,
                    Err(_) => todo!()
                };
                let tls_acceptor = tls_acceptor.clone();
                let mut exec_svc_clone = exec_svc.clone();
                exec_svc_clone.set_src(remote_addr);
                tokio::spawn(async move {
                    match tls_acceptor.accept(tcp_stream).await {
                        Ok(tls_stream) => {
                            let io = TokioIo::new(tls_stream);

                            if let Err(err) = auto::Builder::new(ServiceExecutor).serve_connection(io, exec_svc_clone).await {
                                eprintln!("failed to serve connection: {:#}", err);
                            }
                        },
                        Err(err) => {
                            eprintln!("failed to perform tls handshake: {err:#?}");
                        }
                    };
                });
            }
        })

    }

}