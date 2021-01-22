#![allow(clippy::identity_op)]

mod type_marker;

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

pub mod custom_encoder;
pub mod read;
pub mod write;
