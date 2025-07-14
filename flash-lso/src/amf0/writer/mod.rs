mod amf0_writer;
mod array_writer;
mod cache_key;
mod obj_writer;
mod object_writer;

pub use amf0_writer::Amf0Writer;
pub use array_writer::ArrayWriter;
pub use cache_key::CacheKey;
pub use obj_writer::ObjWriter;
pub use object_writer::ObjectWriter;

#[test]
fn fff() {
    let mut w = Amf0Writer::default();
    let (aw, _) = w.object(CacheKey::from_ptr(std::ptr::null::<u8>()));
    let mut aw = aw.unwrap();
    {
        aw.string("asdf", "asfd");
        {
            let (aw2, _) = aw.object(CacheKey::from_ptr(core::ptr::dangling::<u8>()));
            let mut aw2 = aw2.unwrap();
            aw2.string("asf", "asdf");
            aw2.commit("asf");
        }
    }
    aw.commit("foo");
}
