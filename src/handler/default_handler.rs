use std::future::Future;
use std::pin::Pin;
use hyper::body::Buf;
use crate::exchange::Exchange;
use crate::handler::Handler;

#[derive(Debug, Clone, Default)]
pub struct DefaultHandler;
impl Handler for DefaultHandler
{

    fn process<'i1, 'i2, 'o>(&'i1 self, context: &'i2 mut Exchange) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>> where 'i1: 'o, 'i2: 'o, Self: 'o {
        Box::pin(async move {
//            let consumed = context.consume_request_context();
//            Ok(Exchange::new(consumed.0, consumed.3, hyper::Response::default(), consumed.2))
            todo!()
        })
    }
}