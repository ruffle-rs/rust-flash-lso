use crate::amf3::read::AMF3Decoder;

use crate::amf3::write::AMF3Encoder;

use crate::types::Element;
use crate::types::*;

use crate::nom_utils::AMFResult;

/// A trait to define encoding for custom types for use with Externalized objects
pub trait CustomEncoder {
    /// This should implement the encoding of a given set of external elements for the given class definition
    /// Access to the AMF3Encoder is given to allow access to caches
    /// This implements the encoding side of externalized type support
    fn encode(
        &self,
        elements: &[Element],
        class_def: &Option<ClassDefinition>,
        encoder: &AMF3Encoder,
    ) -> Vec<u8>;
}

/// A trait to define decoding for custom types for use with Externalized objects
pub trait CustomDecoder {
    /// This should implement the decoding of a given set of external elements
    /// Access to the AMF3Decoder is given to allow access to caches
    /// This implements the decoding side of externalized type support
    fn decode<'a>(&self, i: &'a [u8], dec: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>>;
}
