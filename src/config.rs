use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::future::Future;
use std::io::Read;
use std::marker;
use std::pin::Pin;
use std::str::FromStr;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::{Buf, Bytes};
use serde_json::Value;
use serde::Deserialize;
use crate::exchange::Exchange;
use crate::handler::exchange_trace_handler::{ChainExecutionStartHandler, ChainExecutionStopHandler};
use crate::handler::{Handler};
use crate::handler::default_handler::DefaultHandler;
use crate::handler::request_echo_handler::RequestEchoHandler;
use crate::handler::reverse_proxy_handler::{ProxyConfig, ReverseProxyHandler};

type ConfigError = Box<dyn std::error::Error>;

#[derive(Default)]
pub struct HandlerRegister {
    registered: HashMap<String, Box<dyn Handler>>
}

impl HandlerRegister
{
    pub fn register(&mut self, handler_name: String, handler_instance: Box<dyn Handler>) -> Result<(), ConfigError> {
        if !self.registered.contains_key(&handler_name) {

        }
        Err(DuplicateHandlerError.into())
    }
}

#[derive(Debug, Clone)]
struct DuplicateHandlerError;

impl Display for DuplicateHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duplicate handler defined.")
    }
}
impl std::error::Error for DuplicateHandlerError {}

#[derive(Deserialize, Debug, Clone, PartialOrd, PartialEq, Default)]
pub enum HttpMethod {

    #[serde(alias = "OPTIONS")]
    Options,

    #[serde(alias = "GET")]
    #[default]
    Get,

    #[serde(alias = "POST")]
    Post,

    #[serde(alias = "PUT")]
    Put,

    #[serde(alias = "DELETE")]
    Delete,

    #[serde(alias = "HEAD")]
    Head,

    #[serde(alias = "TRACE")]
    Trace,

    #[serde(alias = "CONNECT")]
    Connect,

    #[serde(alias = "PATCH")]
    Patch
}

//pub enum Registry {
//    /* request handlers */
//    Default(DefaultHandler),
//    ProxyHandler(ReverseProxyHandler),
//    RequestEchoHandler(RequestEchoHandler),
//
//    /* request middleware */
//    ChainExecutionStartHandler(ChainExecutionStartHandler),
//
//    /* response middleware */
//    ChainExecutionStopHandler(ChainExecutionStopHandler),
//
//}
//
//impl Default for Registry {
//    fn default() -> Self {
//        Registry::Default(DefaultHandler::default())
//    }
//}
//
//impl FromStr for Registry {
//    type Err = ();
//
//    fn from_str(s: &str) -> Result<Self, Self::Err> {
//        match s.to_lowercase().as_str() {
//            "chainexecutionstophandler" => Ok(Registry::ChainExecutionStopHandler(ChainExecutionStopHandler::default())),
//            "requestechohandler" => Ok(Registry::RequestEchoHandler(RequestEchoHandler::default())),
//            "chainexecutionstarthandler" => Ok(Registry::ChainExecutionStartHandler(ChainExecutionStartHandler::default())),
//            "reverseproxyhandler"=> {
//                let proxy_config = ProxyConfig::load("./config/proxy.json");
//                let proxy_handler = ReverseProxyHandler::new(proxy_config.unwrap());
//                Ok(Registry::ProxyHandler(proxy_handler))
//            },
//            _ => Err(())
//        }
//    }
//}
//
//impl Registry
//{
//    pub fn get(&self) -> Result<&dyn Handler, ()> {
//        match self {
//            Registry::ChainExecutionStartHandler(handler) => Ok(handler),
//            |_ => Err(())
//        }
//    }
//}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "options" => Ok(HttpMethod::Options),
            "get" => Ok(HttpMethod::Get),
            "post" => Ok(HttpMethod::Post),
            "put" => Ok(HttpMethod::Put),
            "delete" => Ok(HttpMethod::Delete),
            "head" => Ok(HttpMethod::Head),
            "connect" => Ok(HttpMethod::Connect),
            "patch" => Ok(HttpMethod::Patch),
            _ => Err(())
        }
    }
}

#[derive(Deserialize, Default)]
pub struct PathConfig {
    pub path: String,
    pub method: HttpMethod,
    pub request: Vec<Box<dyn Handler>>,
    pub response: Vec<Box<dyn Handler>>,
}

#[derive(Deserialize, Default)]
pub struct ServerServiceConfig {
    pub paths: Vec<PathConfig>,
}

//fn register_handlers(registered_handlers: &mut Vec<Registry>, handler_strings: &Vec<String>) -> Result<(), ()> {
//    for handler_string in handler_strings {
//        match Registry::from_str(handler_string) {
//            Ok(handler) => {
//                registered_handlers.push(handler);
//            },
//            Err(e) => return Err(e)
//        }
//    }
//    Ok(())
//}

//pub fn load_from_file(path: String) -> Option<ServerServiceConfig> {
//    if let Ok(mut file) = File::open(path) {
//        let mut contents = String::new();
//        if let Err(e) = file.read_to_string(&mut contents) {
//            panic!("File contains non-UTF-8 characters: {}", e)
//        }
//        if let Ok(config) = serde_json::from_str(&contents) {
//            let mut server_service_config: ServerServiceConfig = config;
//            for path_config in &mut server_service_config.paths {
//                let mut request_middleware: Vec<Registry> = vec![];
//                if let Some(chain) = path_config.request.middleware.as_ref() {
//                    match register_handlers(&mut request_middleware, chain) {
//                        Ok(_) => {},
//                        Err(_) => panic!("Could not register request chain middleware for path: {}", path_config.path)
//                    }
//                }
//                path_config.request.loaded_middleware = request_middleware;
//                let request_handler = match path_config.request.handler.as_ref() {
//                    None => Registry::Default(DefaultHandler::default()),
//                    Some(handler) => {
//                        match Registry::from_str(handler) {
//                            Ok(handler) => handler,
//                            Err(_) => panic!("Could not register request handler for path: {}", path_config.path)
//                        }
//                    }
//                };
//                path_config.request.loaded_handler = request_handler;
//
//                let mut response_middleware: Vec<Registry> = vec![];
//                if let Some(chain) = path_config.response.middleware.as_ref() {
//                    match register_handlers(&mut response_middleware, chain) {
//                        Ok(_) => {},
//                        Err(_) => panic!("Could not register response chain handlers for path: {}", path_config.path)
//                    }
//                }
//                path_config.response.loaded_middleware = response_middleware;
//                let response_handler = match path_config.response.handler.as_ref() {
//                    None => Registry::Default(DefaultHandler::default()),
//                    Some(handler) => {
//                        match Registry::from_str(handler) {
//                            Ok(handler) => handler,
//                            Err(_) => panic!("Could not register response handler for path: {}", path_config.path)
//                        }
//                    }
//                };
//                path_config.response.loaded_handler = response_handler;
//            }
//            return Some(server_service_config);
//        }
//    }
//    return None;
//}

mod test {
    use super::*;

    #[derive(Debug, Clone, Default)]
    pub struct TestRegisterHandler;
    impl Handler for TestRegisterHandler
    {

        fn process<'i1, 'i2, 'o>(&'i1 self, context: &'i2 mut Exchange) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>> where 'i1: 'o, 'i2: 'o, Self: 'o {
            todo!()
        }
    }

    #[test]
    fn test_register() {
        let register = HandlerRegister::default();
        //let test_middleware =
    }
}