use crate::amf3::read::AMF3Decoder;
use crate::amf3::type_marker::TypeMarker;
use crate::amf3::write::AMF3Encoder;
use crate::length::Length;
use crate::types::*;
use crate::types::{Element, Value};
use crate::PADDING;
use enumset::EnumSet;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::error::{make_error, ErrorKind};
use nom::lib::std::collections::HashMap;
use nom::multi::{many_m_n, separated_list0};
use nom::number::complete::{be_f64, be_i32, be_u32, be_u8};
use nom::take;
use nom::take_str;
use nom::Err;
use nom::IResult;
use std::convert::{TryFrom, TryInto};
use std::ops::DerefMut;
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
    Rc<Box<dyn for<'a> Fn(&'a [u8], &mut AMF3Decoder) -> IResult<&'a [u8], Vec<Element>>>>;
