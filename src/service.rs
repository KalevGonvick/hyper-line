use std::future::Future;
#[derive(Clone)]
pub struct ServiceExecutor;

impl<F> hyper::rt::Executor<F> for ServiceExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}