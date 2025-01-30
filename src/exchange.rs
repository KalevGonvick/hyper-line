use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use crate::attachment_key::AttachmentKey;
use crate::callback::Callback;
use crate::status::Status;
use http_body_util::BodyExt;

pub struct Exchange
{
    status: Status,
    src: SocketAddr,
    request: Request<UnsyncBoxBody<Bytes, Infallible>>,
    response: Response<UnsyncBoxBody<Bytes, Infallible>>,
    request_listeners: Vec<Callback<Self>>,
    response_listeners: Vec<Callback<Self>>,
    attachments: HashMap<(AttachmentKey, TypeId), Box<dyn Any + Send>>
}

impl Exchange {

    pub fn new(src: SocketAddr) -> Self {
        log::debug!("Building new exchange for client: {}", src);
        Self {
            status: Status(200),
            src,
            request: Request::default(),
            response: Response::default(),
            request_listeners: vec![],
            response_listeners: vec![],
            attachments: HashMap::new()
        }
    }
    pub fn add_attachment<K>(
        &mut self,
        key: AttachmentKey,
        value: Box<dyn Any + Send>
    ) where K: 'static + Send,
    {
        let type_id = TypeId::of::<K>();
        self.attachments.insert((key, type_id), value);
    }

    pub fn attachment<K>(
        &self,
        key: AttachmentKey
    ) -> Option<&K>
    where K: 'static + Send,
    {
        let type_id = TypeId::of::<K>();
        if let Some(option_any) = self.attachments.get(&(key, type_id)) {
            option_any.downcast_ref::<K>()
        } else {
            None
        }
    }

    pub fn attachment_mut<K>(
        &mut self,
        key: AttachmentKey
    ) -> Option<&mut K>
    where
        K: 'static + Send,
    {
        let type_id = TypeId::of::<K>();
        if let Some(option_any) = self.attachments.get_mut(&(key, type_id)) {
            option_any.downcast_mut::<K>()
        } else {
            None
        }
    }

    pub fn add_request_listener(
        &mut self,
        callback: impl Fn(Box<&Self>) + Send + 'static
    ) where
        Self: Send,
        Self: Sized
    {
        self.request_listeners.push(Callback::new(callback))
    }

    pub fn add_response_listener(
        &mut self,
        callback: impl Fn(Box<&Self>) + Send + 'static
    )
    where
        Self: Send,
        Self: Sized
    {
        self.response_listeners.push(Callback::new(callback))
    }

    fn execute_request_listeners(
        &mut self
    ) -> Result<(), ()>
    {
        if self.status.all_flags_clear(Status::REQUEST_LISTENERS_COMPLETE) {
            self.status |= Status::REQUEST_LISTENERS_COMPLETE;
            return self.execute_callbacks(&self.request_listeners);
        }

        log::error!("Request listeners have already been executed.");
        Err(())
    }

    fn execute_response_listeners(
        &mut self
    ) -> Result<(), ()>
    {
        if self.status.all_flags_clear(Status::RESPONSE_LISTENERS_COMPLETE) {
            self.status |= Status::RESPONSE_LISTENERS_COMPLETE;
            return self.execute_callbacks(&self.response_listeners);
        }

        log::error!("Response listeners have already been executed.");
        Err(())
    }

    fn execute_callbacks(
        &self,
        callbacks: &Vec<Callback<Self>>
    ) -> Result<(), ()>
    where
        Self: Send
    {
        let mut pos = 0usize;
        while !callbacks.is_empty() && pos < callbacks.len() {
            log::trace!("Executing callback {}", pos);
            match callbacks.get(pos) {
                Some(callback) => callback.invoke(Box::new(self)),
                None => return Err(())
            }
            pos += 1;
        }
        Ok(())
    }

    pub async fn buffer_request(
        &mut self,
        request: Request<Incoming>
    ) -> Result<(), ()>
    {
        let (parts, body) = request.into_parts();
        let body = match body.collect().await {
            Ok(x) => x,
            Err(_) => return Err(()),
        }.boxed_unsync();
        let req = Request::from_parts(parts, UnsyncBoxBody::new(body));
        Ok(self.request = req)
    }

    pub fn save_request(
        &mut self,
        request: Request<UnsyncBoxBody<Bytes, Infallible>>
    )
    {
        self.request = request;
    }

    pub fn request(
        &self
    ) -> Result<&Request<UnsyncBoxBody<Bytes, Infallible>>, ()>
    {
        if self.status.all_flags_clear(Status::REQUEST_CONSUMED) {
            return Ok(&self.request);
        }

        log::error!("A request has already been saved for this exchange.");
        Err(())
    }

    pub fn src(&self) -> &SocketAddr {
        &self.src
    }

    pub fn consume_request(
        &mut self
    ) -> Result<Request<UnsyncBoxBody<Bytes, Infallible>>, ()>
    {
        if self.status.all_flags_clear(Status::REQUEST_CONSUMED) {
            self.status |= Status::REQUEST_CONSUMED;
            match self.execute_request_listeners() {
                Ok(_) => {
                    log::debug!("Successfully executed request listeners.");
                    let consumed = std::mem::take(&mut self.request);
                    return Ok(consumed);
                }
                Err(_) => todo!()
            }

        }
        Err(())
    }

    pub fn save_response(
        &mut self,
        response: Response<UnsyncBoxBody<Bytes, Infallible>>
    )
    {
        self.response = response;
    }

    pub fn consume_response(
        &mut self
    ) -> Result<Response<UnsyncBoxBody<Bytes, Infallible>>, ()>
    {
        if self.status.all_flags_clear(Status::RESPONSE_CONSUMED) {
            self.status |= Status::RESPONSE_CONSUMED;
            match self.execute_response_listeners() {
                Ok(_) => {
                    log::debug!("Successfully executed response listeners.");
                    let response_code = self.status.0 & Status::RESPONSE_CODE_BITMASK;
                    *self.response.status_mut() = hyper::StatusCode::from_u16(response_code as u16).unwrap();
                    log::debug!("Response code: {}", response_code);
                    let consumed = std::mem::take(&mut self.response);
                    return Ok(consumed);
                },
                Err(_) => todo!()
            }

        }
        Err(())
    }

    pub fn status(&self) -> &Status {
        &self.status
    }
}