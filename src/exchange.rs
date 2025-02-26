use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use http_body_util::combinators::UnsyncBoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use crate::attachment_key::AttachmentKey;
use crate::callback::Callback;
use crate::status::Status;
use http_body_util::BodyExt;
use crate::config::ServerConfig;

pub struct Exchange<I, O>
where
    I: Default + Send + 'static,
    O: Default + Send + 'static,
{
    status: Status,
    input: I,
    output: O,
    input_listeners: Vec<Callback<Self>>,
    output_listeners: Vec<Callback<Self>>,
    attachments: HashMap<(AttachmentKey, TypeId), Box<dyn Any + Send>>
}

impl<I, O> Exchange<I, O>
where
    I: Default + Send + 'static,
    O: Default + Send + 'static
{

    pub fn new() -> Self {
        Self {
            status: Status(200),
            input: I::default(),
            output: O::default(),
            input_listeners: vec![],
            output_listeners: vec![],
            attachments: HashMap::new()
        }
    }
    pub fn add_attachment<K>(
        &mut self,
        key: AttachmentKey,
        value: Box<dyn Any + Send>
    )
    where
        K: 'static + Send,
    {
        let type_id = TypeId::of::<K>();
        self.attachments.insert((key, type_id), value);
    }

    pub fn attachment<K>(
        &self,
        key: AttachmentKey
    ) -> Option<&K>
    where
        K: 'static + Send,
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

    pub fn add_input_listener(
        &mut self,
        callback: impl Fn(Box<&Self>) + Send + 'static
    )
    where
        Self: Send,
        Self: Sized
    {
        self.input_listeners.push(Callback::new(callback))
    }

    pub fn add_output_listener(
        &mut self,
        callback: impl Fn(Box<&Self>) + Send + 'static
    )
    where
        Self: Send,
        Self: Sized
    {
        self.output_listeners.push(Callback::new(callback))
    }

    fn execute_input_listeners(
        &mut self
    ) -> Result<(), ()>
    {
        if self.status.all_flags_clear(Status::REQUEST_LISTENERS_COMPLETE) {
            self.status |= Status::REQUEST_LISTENERS_COMPLETE;
            return self.execute_callbacks(&self.input_listeners);
        }

        log::error!("Request listeners have already been executed.");
        Err(())
    }

    fn execute_output_listeners(
        &mut self
    ) -> Result<(), ()>
    {
        if self.status.all_flags_clear(Status::RESPONSE_LISTENERS_COMPLETE) {
            self.status |= Status::RESPONSE_LISTENERS_COMPLETE;
            return self.execute_callbacks(&self.output_listeners);
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

    pub fn save_input(
        &mut self,
        request: I
    )
    {
        self.input = request;
    }

    pub fn input(
        &self
    ) -> Result<&I, ()>
    {
        if self.status.all_flags_clear(Status::REQUEST_CONSUMED) {
            return Ok(&self.input);
        }

        log::error!("A request has already been saved for this exchange.");

        Err(())
    }

    pub fn consume_request(
        &mut self
    ) -> Result<I, ()>
    {
        if self.status.all_flags_clear(Status::REQUEST_CONSUMED) {
            self.status |= Status::REQUEST_CONSUMED;
            match self.execute_input_listeners() {
                Ok(_) => {

                    log::debug!("Successfully executed request listeners.");

                    let consumed = std::mem::take(&mut self.input);
                    return Ok(consumed);
                }
                Err(_) => todo!()
            }

        }
        Err(())
    }

    pub fn save_output(
        &mut self,
        response: O
    )
    {
        self.output = response;
    }

    pub fn consume_output(
        &mut self
    ) -> Result<O, ()>
    {
        if self.status.all_flags_clear(Status::RESPONSE_CONSUMED) {
            self.status |= Status::RESPONSE_CONSUMED;
            match self.execute_output_listeners() {
                Ok(_) => {

                    log::debug!("Successfully executed response listeners.");

                    let response_code = self.status.0 & Status::RESPONSE_CODE_BITMASK;
                    //*self.response.status_mut() = hyper::StatusCode::from_u16(response_code as u16).unwrap();

                    log::debug!("Response code: {}", response_code);

                    let consumed = std::mem::take(&mut self.output);
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