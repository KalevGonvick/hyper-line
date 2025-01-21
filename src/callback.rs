pub(crate) struct Callback<T: Send + ?Sized> {
    callback: Box<dyn Fn(Box<&T>) + Send + 'static>
}
impl<T: Send + ?Sized> Callback<T> {
    pub fn new(
        callback: impl Fn(Box<&T>) + Send + 'static
    ) -> Self
    {
        Self { callback: Box::new(callback) }
    }

    pub fn invoke(
        &self,
        context: Box<&T>
    )
    {
        (self.callback)(context);
    }
}