pub mod decode {
    use crate::amf3::AMF3Decoder;
    use crate::types::SolValue;
    use nom::number::complete::be_u8;
    use nom::IResult;

    const NEXT_FLAG: u8 = 128;
    const BODY_FLAG: u8 = 1;
    const CLIENT_ID_FLAG: u8 = 2;
    const DESTINATION_ID_FLAG: u8 = 4;
    const HEADERS_FLAG: u8 = 8;
    const MESSAGE_ID_FLAG: u8 = 16;
    const TIMESTAMP_FLAG: u8 = 32;
    const TTL_FLAG: u8 = 64;

    const CLIENT_ID_BYTES_FLAG: u8 = 1;
    const MESSAGE_ID_BYTES_FLAG: u8 = 2;

    const CORRELATION_ID_FLAG: u8 = 1;
    const CORRELATION_ID_BYTES_FLAG: u8 = 2;

    const OPERATION_FLAG: u8 = 1;

    pub fn parse_abstract_message_flags<'a>(i: &'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
        let mut next_flag = true;
        let mut flags = Vec::new();

        let mut k = i;
        while next_flag {
            let (i, flag) = be_u8(i)?;
            flags.push(flag);
            if flag & 128 == 0 {
                next_flag = false
            }
            k = i;
        }

        Ok((k, flags))
    }

    pub fn parse_abstract_message<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], SolValue> {
        let (i, flags) = parse_abstract_message_flags(i)?;

        let mut k = i;
        for (pos, flags) in flags.iter().enumerate() {
            let mut reserved = 0;

            if pos == 0 {
                if flags & BODY_FLAG != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                if flags & CLIENT_ID_FLAG != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                if flags & DESTINATION_ID_FLAG != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                if flags & HEADERS_FLAG != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                if flags & MESSAGE_ID_FLAG != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                if flags & TIMESTAMP_FLAG != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                if flags & TTL_FLAG != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                reserved = 7;
            } else if pos == 1 {
                if (flags & CLIENT_ID_BYTES_FLAG) != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                if (flags & MESSAGE_ID_BYTES_FLAG) != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }

                reserved = 2;
            }

            if (flags >> reserved) != 0 {
                for j in reserved..6 {
                    if (flags >> j) != 0 {
                        let (j, v) = amf3.parse_single_element(k)?;
                        k = j;
                    }
                }
            }
        }

        Ok((i, SolValue::Null))
    }

    pub fn parse_async_message<'a>(i: &'a [u8], amf3: &AMF3Decoder) -> IResult<&'a [u8], SolValue> {
        let (i, msg) = parse_abstract_message(i, amf3)?;

        let (i, flags) = parse_abstract_message_flags(i)?;

        let mut k = i;
        for (pos, flags) in flags.iter().enumerate() {
            let mut reserved = 0;
            if pos == 0 {
                if (flags & CORRELATION_ID_FLAG) != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                if (flags & CORRELATION_ID_BYTES_FLAG) != 0 {
                    let (j, v) = amf3.parse_single_element(k)?;
                    k = j;
                }
                reserved = 2;
            }

            if (flags >> reserved) != 0 {
                for j in reserved..6 {
                    if (flags >> j) & 1 != 0 {
                        let (j, v) = amf3.parse_single_element(k)?;
                        k = j;
                    }
                }
            }
        }

        Ok((k, SolValue::Null))
    }

    pub fn parse_acknowledge_message<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], SolValue> {
        let (i, v) = parse_async_message(i, amf3)?;

        let (i, flags) = parse_abstract_message_flags(i)?;

        let mut k = i;
        for flags in flags.iter() {
            if flags != 0 {
                for j in 0..6 {
                    if (flags >> j) & 1 != 0 {
                        let (j, v) = amf3.parse_single_element(k)?;
                        k = j;
                    }
                }
            }
        }

        Ok((k, SolValue::Null))
    }

    pub fn parse_command_message<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], SolValue> {
        let (i, v) = parse_async_message(i, amf3)?;

        let (i, flags) = parse_abstract_message_flags(i)?;

        let mut k = i;
        for (pos, flags) in flags.iter().enumerate() {
            let mut reserved = 0;

            if pos == 0 {
                if (flags & OPERATION_FLAG) != 0 {
                    let (j, op) = amf3.parse_single_element(i)?;
                    k = j;
                }
                reserved = 1;
            }

            if (flags >> reserved) != 0 {
                for j in reserved..6 {
                    if (flags >> j) & 1 != 0 {
                        let (j, v) = amf3.parse_single_element(k)?;
                        k = j;
                    }
                }
            }
        }

        Ok((k, SolValue::Null))
    }

    // arrays
    pub fn parse_array_collection<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], SolValue> {
        amf3.parse_single_element(i)
    }

    pub fn parse_array_list<'a>(i: &'a [u8], amf3: &AMF3Decoder) -> IResult<&'a [u8], SolValue> {
        parse_array_collection(i, amf3)
    }

    // proxies
    pub fn parse_object_proxy<'a>(i: &'a [u8], amf3: &AMF3Decoder) -> IResult<&'a [u8], SolValue> {
        amf3.parse_single_element(i)
    }

    pub fn parse_managed_object_proxy<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], SolValue> {
        parse_object_proxy(i, amf3)
    }

    pub fn parse_serialization_proxy<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], SolValue> {
        amf3.parse_single_element(i)
    }

    pub fn register_decoders(decoder: &mut AMF3Decoder) {
        decoder.external_decoders.insert(
            "flex.messaging.io.AbstractMessage".to_string(),
            Box::new(parse_abstract_message),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.AsyncMessage".to_string(),
            Box::new(parse_async_message),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.AsyncMessageExt".to_string(),
            Box::new(parse_async_message),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.AcknowledgeMessage".to_string(),
            Box::new(parse_acknowledge_message),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.AcknowledgeMessageExt".to_string(),
            Box::new(parse_acknowledge_message),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.CommandMessage".to_string(),
            Box::new(parse_command_message),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.CommandMessageExt".to_string(),
            Box::new(parse_command_message),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.ErrorMessage".to_string(),
            Box::new(parse_acknowledge_message),
        );

        decoder.external_decoders.insert(
            "flex.messaging.io.ArrayCollection".to_string(),
            Box::new(parse_array_collection),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.ArrayList".to_string(),
            Box::new(parse_array_collection),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.ObjectProxy".to_string(),
            Box::new(parse_object_proxy),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.ManagedObjectProxy".to_string(),
            Box::new(parse_managed_object_proxy),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.SerializationProxy".to_string(),
            Box::new(parse_serialization_proxy),
        );
    }
}
