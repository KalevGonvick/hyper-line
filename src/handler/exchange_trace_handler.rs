use std::future::Future;
use std::pin::Pin;
use std::time::{SystemTime};
use log::{error, info};
use crate::exchange::Exchange;
use crate::attachment_key::AttachmentKey;
use crate::handler::{Handler};


#[derive(Debug, Clone, Default)]
pub struct ChainExecutionStartHandler;

#[derive(Debug, Clone, Default)]
pub struct ChainExecutionStopHandler;
pub const TRACE_TIME: AttachmentKey = AttachmentKey(3);
impl<I, O> Handler<I, O> for ChainExecutionStartHandler
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
        Self: 'o
    {
        Box::pin(async move {
            let time = Box::new(SystemTime::now());
            context.add_attachment::<SystemTime>(TRACE_TIME, time);
            Ok(())
        })
    }
}

impl<I, O> Handler<I, O> for ChainExecutionStopHandler
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
        Self: 'o
    {
        Box::pin(async move {
            context.add_response_listener(move |exchange| {
                let trace = match exchange.attachment::<SystemTime>(TRACE_TIME) {
                    None => return,
                    Some(trace) => trace.clone()
                };
                let elapsed = match SystemTime::now().duration_since(trace) {
                    Ok(elapsed) => elapsed,
                    Err(e) => {
                        error!("Failed: {}", e);
                        return;
                    }
                };
                info!("Exchange process duration: {}ms", elapsed.as_millis());
            });
            Ok(())
        })
    }
}