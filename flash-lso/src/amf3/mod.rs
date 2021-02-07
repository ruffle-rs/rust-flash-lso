/// Support for custom encoders / decoders
pub mod custom_encoder;
/// Cache pool for the 3 amf3 cache types
pub mod element_cache;
/// Abstraction over the AMF3 length and reference types
pub mod length;
/// Reading of AMF3 data
pub mod read;
/// AMF3 type markers
mod type_marker;
/// Writing of AMF3 data
pub mod write;
