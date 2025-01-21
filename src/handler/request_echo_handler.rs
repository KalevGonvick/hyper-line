use std::future::Future;
use std::pin::Pin;
use crate::exchange::Exchange;
use crate::handler::Handler;

#[derive(Debug, Clone, Default)]
pub struct RequestEchoHandler;
impl Handler for RequestEchoHandler
{

    fn process<'i1, 'i2, 'o>(&'i1 self, context: &'i2 mut Exchange) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>> where 'i1: 'o, 'i2: 'o {
        Box::pin(async move {
//            let consumed = context.consume_request_context();
//            let res = consumed.1.into_body().collect().await.unwrap().boxed_unsync();
//            Ok(Exchange::new(consumed.0, consumed.3, hyper::Response::new(res), consumed.2))
            todo!()
        })
    }
}