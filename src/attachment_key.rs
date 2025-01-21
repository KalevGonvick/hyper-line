/* I wanted to make this struct use TypeId::of::<>() but it's not stable. */
#[derive(PartialOrd, PartialEq, Hash, Eq)]
pub struct AttachmentKey(pub u32);

impl AttachmentKey {
    /* common attachment keys */
    pub const APP_CONTEXT: AttachmentKey = AttachmentKey(1);
    pub const CACHED_BODY: AttachmentKey = AttachmentKey(2);
}