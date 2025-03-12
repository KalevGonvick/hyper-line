use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

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
    custom_listeners: Vec<Callback<Self>>,
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
            custom_listeners: vec![],
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

    pub fn add_custom_listener(
        &mut self,
        callback: impl Fn(Box<&Self>) + Send + 'static
    )
    where
        Self: Send + Sized
    {
        self.custom_listeners.push(Callback::new(callback));
    }

    pub fn add_input_listener(
        &mut self,
        callback: impl Fn(Box<&Self>) + Send + 'static
    )
    where
        Self: Send + Sized
    {
        self.input_listeners.push(Callback::new(callback))
    }

    pub fn add_output_listener(
        &mut self,
        callback: impl Fn(Box<&Self>) + Send + 'static
    )
    where
        Self: Send + Sized
    {
        self.output_listeners.push(Callback::new(callback))
    }

    fn execute_custom_listeners(&mut self) -> Result<(), ()> {
        if self.status.all_flags_clear(Status::CUSTOM_LISTENERS_COMPLETE) {
            self.status |= Status::CUSTOM_LISTENERS_COMPLETE;
            return self.execute_callbacks(&self.custom_listeners);
        }

        log::error!("Custom listeners have already been executed.");
        Err(())
    }

    fn execute_input_listeners(
        &mut self
    ) -> Result<(), ()>
    {
        if self.status.all_flags_clear(Status::INPUT_LISTENERS_COMPLETE) {
            self.status |= Status::INPUT_LISTENERS_COMPLETE;
            return self.execute_callbacks(&self.input_listeners);
        }

        log::error!("Request listeners have already been executed.");
        Err(())
    }

    fn execute_output_listeners(
        &mut self
    ) -> Result<(), ()>
    {
        if self.status.all_flags_clear(Status::OUTPUT_LISTENERS_COMPLETE) {
            self.status |= Status::OUTPUT_LISTENERS_COMPLETE;
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
        if self.status.all_flags_clear(Status::INPUT_CONSUMED) {
            return Ok(&self.input);
        }

        log::error!("A request has already been saved for this exchange.");

        Err(())
    }

    pub fn consume_request(
        &mut self
    ) -> Result<I, ()>
    {
        if self.status.all_flags_clear(Status::INPUT_CONSUMED) {
            self.status |= Status::INPUT_CONSUMED;
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
        if self.status.all_flags_clear(Status::OUTPUT_CONSUMED) {
            self.status |= Status::OUTPUT_CONSUMED;
            match self.execute_output_listeners() {
                Ok(_) => {

                    log::debug!("Successfully executed response listeners.");

                    let response_code = self.status.0 & Status::STATUS_CODE_BITMASK;
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

/* I wanted to make this struct use TypeId::of::<>() but it's not stable. */
#[derive(PartialOrd, PartialEq, Hash, Eq)]
pub struct AttachmentKey(pub u32);

impl AttachmentKey {
    /* common attachment keys */
    pub const APP_CONTEXT: AttachmentKey = AttachmentKey(1);
    pub const CLIENT_SRC: AttachmentKey = AttachmentKey(2);
    pub const CACHED_BODY: AttachmentKey = AttachmentKey(3);
}

pub struct Callback<T: Send + ?Sized> {
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

#[derive(Clone,Copy)]
pub struct Status(pub i32);

#[inline]
pub const fn i32_bit_mask(low: i32, high: i32) -> i32 {
    assert!(low >= 0);
    assert!(low <= high);
    assert!(high < 32);
    if high == 31 {
        0
    } else {
        (1 << high + 1) - (1 << low)
    }
}

impl Status {
    pub const STATUS_CODE_BITMASK: i32 = i32_bit_mask(0, 9);
    pub const INPUT_CONSUMED: Self = Self(1 << 10);
    pub const OUTPUT_CONSUMED: Self = Self(1 << 11);
    pub const INPUT_LISTENERS_COMPLETE: Self = Self(1 << 12);
    pub const OUTPUT_LISTENERS_COMPLETE: Self = Self(1 << 13);
    pub const CUSTOM_LISTENERS_COMPLETE: Self = Self(1 << 14);
    pub const INPUT_BUFFERED: Self = Self(1 << 15);
    pub const OUTPUT_BUFFERED: Self = Self(1 << 16);

    pub fn any_flags(&self, flags: Status) -> bool {
        self.0 & flags.0 != 0
    }

    pub fn any_flags_clear(&self, flags: Status) -> bool {
        self.0 & flags.0 != flags.0
    }

    pub fn all_flags(&self, flags: Status) -> bool {
        self.0 & flags.0 == 0
    }

    pub fn all_flags_clear(&self, flags: Status) -> bool {
        self.0 & flags.0 == 0
    }
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl BitOrAssign for Status {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAndAssign for Status {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl Not for Status {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAnd for Status {
    type Output = Self;
    fn bitand(
        self,
        rhs: Self
    ) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Status {
    type Output = Self;
    fn bitor(
        self,
        rhs: Self
    ) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod test {
    use log::info;
    use super::*;

    const TEST_ATTACHMENT: AttachmentKey = AttachmentKey(1);

    #[test]
    fn test_exchange_attachments() {
        let mut ex: Exchange<usize, usize> = Exchange::new();
        ex.add_attachment::<String>(TEST_ATTACHMENT, Box::new(String::from("This is a test value for the test attachment.")));
        assert_eq!(ex.attachments.len(), 1);

        match ex.attachment::<String>(TEST_ATTACHMENT) {
            None => assert!(false),
            Some(test_attachment) => {
                assert_eq!(test_attachment, "This is a test value for the test attachment.");
            }
        }
    }

    #[test]
    fn test_custom_listener() {
        let mut ex: Exchange<usize, usize> = Exchange::new();
        ex.add_custom_listener(|ex| {
            info!("This is a custom listener executing...");
        });

        match ex.execute_custom_listeners() {
            Ok(_) => assert!(true),
            Err(_) => assert!(false, "Should execute custom listeners the first time.")
        }

        match ex.execute_custom_listeners() {
            Ok(_) => assert!(false, "Should NOT execute custom listeners the second time."),
            Err(_) => assert!(true),
        }
    }
}