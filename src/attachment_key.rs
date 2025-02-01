/* I wanted to make this struct use TypeId::of::<>() but it's not stable. */
#[derive(PartialOrd, PartialEq, Hash, Eq)]
pub struct AttachmentKey(pub u32);
//pub struct AttachmentKey2<T>(pub u32, T);


impl AttachmentKey {
    /* common attachment keys */
    pub const APP_CONTEXT: AttachmentKey = AttachmentKey(1);
    pub const CACHED_BODY: AttachmentKey = AttachmentKey(2);
}

//impl<T> AttachmentKey2<T> {
//    pub const CACHED_BODY: AttachmentKey2<Bytes> = AttachmentKey2(1, Bytes::default().type_id());
//
//
//    pub fn hash_key(&self) -> (u32, TypeId) {
//        (self.0, TypeId::of::<T>())
//    }
//}
