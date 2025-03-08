pub mod reverse_proxy_handler;
pub mod exchange_trace_handler;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, LazyLock, RwLock};
use crate::exchange::Exchange;
use crate::{HttpBody, HttpRequest, HttpResponse};

pub type HttpHandler = Box<dyn Handler<HttpBody, HttpBody> + Send + Sync + 'static>;

static REGISTERED_HANDLERS: LazyLock<RwLock<HashMap<HandlerId, Arc<dyn Handler<HttpRequest, HttpResponse> + Sync + Send + 'static>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));
pub fn register(id: &str, handler: Arc<dyn Handler<HttpRequest, HttpResponse> + Sync + Send + 'static>) {
    REGISTERED_HANDLERS.write().unwrap().insert(HandlerId(id.to_string()), handler);
}

pub(crate) fn get_handler(id: HandlerId) -> Option<Arc<dyn Handler<HttpRequest, HttpResponse> + Sync + Send + 'static>> {
    REGISTERED_HANDLERS.read().unwrap().get(&id).cloned()
}
#[derive(Eq, Hash, PartialEq)]
pub struct HandlerId(pub String);

pub trait Handler<I, O>: Send
where
    I: Default + Send + 'static,
    O: Default + Send + 'static,
{
    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        context: &'i2 mut Exchange<I, O>
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o;
}



