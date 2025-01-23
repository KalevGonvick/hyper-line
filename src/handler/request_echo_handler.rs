use std::future::Future;
use std::pin::Pin;
use hyper::http::response::Parts;
use hyper::Response;
use log::info;
use crate::exchange::Exchange;
use crate::handler::Handler;

#[derive(Debug, Clone, Default)]
pub struct RequestEchoHandler;
impl Handler for RequestEchoHandler
{

    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        context: &'i2 mut Exchange
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o
    {
        Box::pin(async move {
            info!("Echo handler");
            let consumed = context.consume_request().unwrap();
            let (_, request) = consumed.into_parts();
            let echoed_response = Response::new(request);
            context.save_response(echoed_response).await;
            Ok(())
        })
    }
}