//! Handles decoding of flex types

use crate::amf3::read::AMF3Decoder;
use crate::extra::flex::{
    BODY_FLAG, CLIENT_ID_BYTES_FLAG, CLIENT_ID_FLAG, CORRELATION_ID_BYTES_FLAG,
    CORRELATION_ID_FLAG, DESTINATION_ID_FLAG, HEADERS_FLAG, MESSAGE_ID_BYTES_FLAG, MESSAGE_ID_FLAG,
    NEXT_FLAG, OPERATION_FLAG, TIMESTAMP_FLAG, TTL_FLAG,
};
use crate::nom_utils::AMFResult;
use crate::types::Element;
use nom::number::complete::be_u8;
use crate::amf3::custom_encoder::CustomDecoder;

fn parse_abstract_message_flags(i: &[u8]) -> AMFResult<'_, Vec<u8>> {
    let mut next_flag = true;
    let mut flags = Vec::new();

    let mut k = i;
    while next_flag {
        let (i, flag) = be_u8(i)?;
        flags.push(flag);
        if flag & NEXT_FLAG == 0 {
            next_flag = false
        }
        k = i;
    }

    Ok((k, flags))
}

fn parse_abstract_message<'a>(i: &'a [u8], amf3: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
    let (i, flags) = parse_abstract_message_flags(i)?;

    let mut elements = Vec::new();

    let mut k = i;
    for (pos, flags) in flags.iter().enumerate() {
        let mut reserved = 0;

        if pos == 0 {
            if flags & BODY_FLAG != 0 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "body".to_string(),
                    value,
                });
                k = j;
            }
            if flags & CLIENT_ID_FLAG != 0 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "client_id".to_string(),
                    value,
                });
                k = j;
            }
            if flags & DESTINATION_ID_FLAG != 0 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "destination".to_string(),
                    value,
                });
                k = j;
            }
            if flags & HEADERS_FLAG != 0 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "headers".to_string(),
                    value,
                });
                k = j;
            }
            if flags & MESSAGE_ID_FLAG != 0 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "message_id".to_string(),
                    value,
                });
                k = j;
            }
            if flags & TIMESTAMP_FLAG != 0 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "timestamp".to_string(),
                    value,
                });
                k = j;
            }
            if flags & TTL_FLAG != 0 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "ttl".to_string(),
                    value,
                });
                k = j;
            }
            reserved = 7;
        } else if pos == 1 {
            if (flags & CLIENT_ID_BYTES_FLAG) != 0 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "client_id_bytes".to_string(),
                    value,
                });
                k = j;
            }
            if (flags & MESSAGE_ID_BYTES_FLAG) != 0 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "message_id_bytes".to_string(),
                    value,
                });
                k = j;
            }

            reserved = 2;
        }

        if (flags >> reserved) != 0 {
            for j in reserved..6 {
                if (flags >> j) != 0 {
                    let (jj, value) = amf3.parse_single_element(k)?;
                    elements.push(Element {
                        name: format!("children_{j}"),
                        value,
                    });
                    k = jj;
                }
            }
        }
    }

    Ok((i, elements))
}

fn parse_async_message<'a>(i: &'a [u8], amf3: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
    let (i, msg) = parse_abstract_message(i, amf3)?;

    let (i, flags) = parse_abstract_message_flags(i)?;

    let mut elements = msg;

    let mut k = i;
    for (pos, flags) in flags.iter().enumerate() {
        let mut reserved = 0;
        if pos == 0 {
            if (flags & CORRELATION_ID_FLAG) != 0u8 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "correlation_id".to_string(),
                    value,
                });
                k = j;
            }
            if (flags & CORRELATION_ID_BYTES_FLAG) != 0u8 {
                let (j, value) = amf3.parse_single_element(k)?;
                elements.push(Element {
                    name: "correlation_id_bytes".to_string(),
                    value,
                });
                k = j;
            }
            reserved = 2;
        }

        if (flags >> reserved) != 0u8 {
            for j in reserved..6 {
                if (flags >> j) & 1 != 0u8 {
                    let (jj, value) = amf3.parse_single_element(k)?;
                    elements.push(Element {
                        name: format!("children_async_{j}"),
                        value,
                    });
                    k = jj;
                }
            }
        }
    }

    Ok((k, elements))
}

fn parse_acknowledge_message<'a>(
    i: &'a [u8],
    amf3: &mut AMF3Decoder,
) -> AMFResult<'a, Vec<Element>> {
    let (i, msg) = parse_async_message(i, amf3)?;

    let (i, flags) = parse_abstract_message_flags(i)?;

    let mut elements = msg;

    let mut k = i;
    for flags in flags.iter() {
        if *flags != 0 {
            for j in 0..6 {
                if (flags >> j) & 1 != 0 {
                    let (jj, value) = amf3.parse_single_element(k)?;
                    elements.push(Element {
                        name: format!("children_acknowledge_{j}"),
                        value,
                    });
                    k = jj;
                }
            }
        }
    }

    Ok((k, elements))
}

fn parse_command_message<'a>(i: &'a [u8], amf3: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
    let (i, msg) = parse_async_message(i, amf3)?;

    let (i, flags) = parse_abstract_message_flags(i)?;

    let mut elements = msg;

    let mut k = i;
    for (pos, flags) in flags.iter().enumerate() {
        let mut reserved = 0;

        if pos == 0 {
            if (flags & OPERATION_FLAG) != 0 {
                let (j, value) = amf3.parse_single_element(i)?;
                elements.push(Element {
                    name: "operation".to_string(),
                    value,
                });
                k = j;
            }
            reserved = 1;
        }

        if (flags >> reserved) != 0 {
            for j in reserved..6 {
                if (flags >> j) & 1 != 0 {
                    let (jj, value) = amf3.parse_single_element(k)?;
                    elements.push(Element {
                        name: format!("children_command_{j}"),
                        value,
                    });
                    k = jj;
                }
            }
        }
    }

    Ok((k, elements))
}

// all arrays
fn parse_array_collection<'a>(i: &'a [u8], amf3: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
    let (i, value) = amf3.parse_single_element(i)?;

    let el = vec![Element {
        name: "data".to_string(),
        value,
    }];

    Ok((i, el))
}

// all proxies
fn parse_object_proxy<'a>(i: &'a [u8], amf3: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
    let (i, value) = amf3.parse_single_element(i)?;

    let el = vec![Element {
        name: "object".to_string(),
        value,
    }];

    Ok((i, el))
}

/// Register the flex decoders into the given AMF3Decoder
#[inline]
pub fn register_decoders(decoder: &mut AMF3Decoder) {
    // TODOS:
    // Value::Object{id, objval}
    // remove comments
    // no unwrap

    #[derive(Default)]
    struct FlexAbstractMessageParser;
    impl CustomDecoder for FlexAbstractMessageParser {
        fn decode<'a>(&self, i: &'a [u8], dec: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
            parse_abstract_message(i, dec)
        }
    }
    decoder.register_custom_decoder::<FlexAbstractMessageParser>("flex.messaging.io.AbstractMessage");

    #[derive(Default)]
    struct FlexAsyncMessageParser;
    impl CustomDecoder for FlexAsyncMessageParser {
        fn decode<'a>(&self, i: &'a [u8], dec: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
            parse_async_message(i, dec)
        }
    }
    decoder.register_custom_decoder::<FlexAsyncMessageParser>("flex.messaging.io.AsyncMessage");
    decoder.register_custom_decoder::<FlexAsyncMessageParser>("flex.messaging.io.AsyncMessageExt");


    #[derive(Default)]
    struct FlexAcknowledgeMessageParser;
    impl CustomDecoder for FlexAcknowledgeMessageParser {
        fn decode<'a>(&self, i: &'a [u8], dec: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
            parse_acknowledge_message(i, dec)
        }
    }
    decoder.register_custom_decoder::<FlexAcknowledgeMessageParser>("flex.messaging.io.AcknowledgeMessage");
    decoder.register_custom_decoder::<FlexAcknowledgeMessageParser>("flex.messaging.io.AcknowledgeMessageExt");
    decoder.register_custom_decoder::<FlexAcknowledgeMessageParser>("flex.messaging.io.ErrorMessage");

    #[derive(Default)]
    struct FlexCommandMessageParser;
    impl CustomDecoder for FlexCommandMessageParser {
        fn decode<'a>(&self, i: &'a [u8], dec: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
            parse_command_message(i, dec)
        }
    }
    decoder.register_custom_decoder::<FlexCommandMessageParser>("flex.messaging.io.CommandMessage");
    decoder.register_custom_decoder::<FlexCommandMessageParser>("flex.messaging.io.CommandMessageExt");


    #[derive(Default)]
    struct FlexArrayCollectionParser;
    impl CustomDecoder for FlexArrayCollectionParser {
        fn decode<'a>(&self, i: &'a [u8], dec: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
            parse_array_collection(i, dec)
        }
    }
    decoder.register_custom_decoder::<FlexArrayCollectionParser>("flex.messaging.io.ArrayCollection");
    decoder.register_custom_decoder::<FlexArrayCollectionParser>("flex.messaging.io.ArrayList");

    #[derive(Default)]
    struct FlexObjectProxyParser;
    impl CustomDecoder for FlexObjectProxyParser {
        fn decode<'a>(&self, i: &'a [u8], dec: &mut AMF3Decoder) -> AMFResult<'a, Vec<Element>> {
            parse_object_proxy(i, dec)
        }
    }
    decoder.register_custom_decoder::<FlexObjectProxyParser>("flex.messaging.io.ObjectProxy");
    decoder.register_custom_decoder::<FlexObjectProxyParser>("flex.messaging.io.ManagedObjectProxy");
    decoder.register_custom_decoder::<FlexObjectProxyParser>("flex.messaging.io.SerializationProxy");
}
