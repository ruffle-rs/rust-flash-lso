use crate::amf3::read::AMF3Decoder;

use crate::amf3::write::AMF3Encoder;

use crate::types::Element;
use crate::types::*;



use crate::nom_utils::AMFResult;
use std::rc::Rc;

/// A trait to define encoding for custom types for use with Externalized objects
pub trait CustomEncoder {
    /// This should implement the encoding of a given set of external elements for the given class definition
    /// Access to the AMF3Encoder is given to allow access to caches
    /// This implements the encoding side of externalized type support
    fn encode<'a>(
        &self,
        elements: &'a [Element],
        class_def: &Option<ClassDefinition>,
        encoder: &AMF3Encoder,
    ) -> Vec<u8>;
}

//TODO: combine with trait
pub type ExternalDecoderFn =
    Rc<Box<dyn for<'a> Fn(&'a [u8], &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>>>>;
