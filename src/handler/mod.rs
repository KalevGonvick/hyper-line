pub mod reverse_proxy_handler;
pub mod exchange_trace_handler;

use std::future::Future;
use std::pin::Pin;
use crate::exchange::Exchange;

pub struct HandlerId(pub String);
pub struct HandlerEntry(HandlerId, Box<dyn Handler + Sync + Send + 'static>);

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

//    fn handler_id(&self) -> HandlerId;
//    fn register_handler(self) -> HandlerEntry
//    where
//        Self: Sync + Send + 'static
//    {
//
//        let id = self.handler_id();
//        HandlerEntry{0: id, 1: Box::new(self)}
//    }




}



