use crate::types::{AMFVersion, Value};
use std::rc::Rc;

/// Reading of AMF Packets
pub mod read;

/// Writing of AMF Packets
pub mod write;

/// An AMF Packet Header
#[derive(Debug, Clone)]
pub struct Header {
    /// The name of this header.
    pub name: String,

    /// If true, the endpoint must immediately abort and error if it does not understand this header.
    pub must_understand: bool,

    /// The value of this header.
    pub value: Rc<Value>,
}

/// An AMF Packet Header
#[derive(Debug, Clone)]
pub struct Message {
    /// The target URI that this message is intended for.
    pub target_uri: String,

    /// The response URI for this message.
    ///
    /// For requests, this should be a unique identifier to represent "this message", for example `/1`.
    /// Responses will target this URI, suffixed with either `/onResult` or `/onStatus` (for success or failure).
    ///
    /// For responses this may be empty.
    pub response_uri: String,

    /// The contents of this message.
    pub contents: Rc<Value>,
}

/// An AMF Packet
#[derive(Debug, Clone)]
pub struct Packet {
    /// The version of this packet. Does not affect serialization.
    pub version: AMFVersion,

    /// Any headers associated with every message inside this packet.
    pub headers: Vec<Header>,

    /// All messages included inside this packet.
    pub messages: Vec<Message>,
}
