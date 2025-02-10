pub mod reverse_proxy_handler;
pub mod exchange_trace_handler;

use std::future::Future;
use std::pin::Pin;
use crate::exchange::Exchange;

pub struct HandlerId(pub String);
//pub struct HandlerEntry(HandlerId, Box<dyn Handler<> + Sync + Send + 'static>);

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



