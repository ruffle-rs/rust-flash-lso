pub mod decode {
    use crate::amf3::AMF3Decoder;
    use crate::types::SolElement;
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

    fn parse_abstract_message_flags<'a>(i: &'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
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
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        let (i, flags) = parse_abstract_message_flags(i)?;

        let mut elements = Vec::new();

        let mut k = i;
        for (pos, flags) in flags.iter().enumerate() {
            let mut reserved = 0;

            if pos == 0 {
                if flags & BODY_FLAG != 0 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
                        name: "body".to_string(),
                        value,
                    });
                    k = j;
                }
                if flags & CLIENT_ID_FLAG != 0 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
                        name: "client_id".to_string(),
                        value,
                    });
                    k = j;
                }
                if flags & DESTINATION_ID_FLAG != 0 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
                        name: "destination".to_string(),
                        value,
                    });
                    k = j;
                }
                if flags & HEADERS_FLAG != 0 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
                        name: "headers".to_string(),
                        value,
                    });
                    k = j;
                }
                if flags & MESSAGE_ID_FLAG != 0 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
                        name: "message_id".to_string(),
                        value,
                    });
                    k = j;
                }
                if flags & TIMESTAMP_FLAG != 0 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
                        name: "timestamp".to_string(),
                        value,
                    });
                    k = j;
                }
                if flags & TTL_FLAG != 0 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
                        name: "ttl".to_string(),
                        value,
                    });
                    k = j;
                }
                reserved = 7;
            } else if pos == 1 {
                if (flags & CLIENT_ID_BYTES_FLAG) != 0 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
                        name: "client_id_bytes".to_string(),
                        value,
                    });
                    k = j;
                }
                if (flags & MESSAGE_ID_BYTES_FLAG) != 0 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
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
                        elements.push(SolElement {
                            name: format!("children_{}", j),
                            value,
                        });
                        k = jj;
                    }
                }
            }
        }

        Ok((i, elements))
    }

    pub fn parse_async_message<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        let (i, msg) = parse_abstract_message(i, amf3)?;

        let (i, flags) = parse_abstract_message_flags(i)?;

        let mut elements = msg;

        let mut k = i;
        for (pos, flags) in flags.iter().enumerate() {
            let mut reserved = 0;
            if pos == 0 {
                if (flags & CORRELATION_ID_FLAG) != 0u8 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
                        name: "correlation_id".to_string(),
                        value,
                    });
                    k = j;
                }
                if (flags & CORRELATION_ID_BYTES_FLAG) != 0u8 {
                    let (j, value) = amf3.parse_single_element(k)?;
                    elements.push(SolElement {
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
                        elements.push(SolElement {
                            name: format!("children_async_{}", j),
                            value,
                        });
                        k = jj;
                    }
                }
            }
        }

        Ok((k, elements))
    }

    pub fn parse_acknowledge_message<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        let (i, msg) = parse_async_message(i, amf3)?;

        let (i, flags) = parse_abstract_message_flags(i)?;

        let mut elements = msg;

        let mut k = i;
        for flags in flags.iter() {
            if *flags != 0 {
                for j in 0..6 {
                    if (flags >> j) & 1 != 0 {
                        let (jj, value) = amf3.parse_single_element(k)?;
                        elements.push(SolElement {
                            name: format!("children_async_{}", j),
                            value,
                        });
                        k = jj;
                    }
                }
            }
        }

        Ok((k, elements))
    }

    pub fn parse_command_message<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        let (i, msg) = parse_async_message(i, amf3)?;

        let (i, flags) = parse_abstract_message_flags(i)?;

        let mut elements = msg;

        let mut k = i;
        for (pos, flags) in flags.iter().enumerate() {
            let mut reserved = 0;

            if pos == 0 {
                if (flags & OPERATION_FLAG) != 0 {
                    let (j, value) = amf3.parse_single_element(i)?;
                    elements.push(SolElement {
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
                        elements.push(SolElement {
                            name: format!("children_command_{}", j),
                            value,
                        });
                        k = jj;
                    }
                }
            }
        }

        Ok((k, elements))
    }

    // arrays
    pub fn parse_array_collection<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        let (i, value) = amf3.parse_single_element(i)?;

        let el = vec![SolElement {
            name: "data".to_string(),
            value,
        }];

        Ok((i, el))
    }

    pub fn parse_array_list<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        parse_array_collection(i, amf3)
    }

    // proxies
    pub fn parse_object_proxy<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        let (i, value) = amf3.parse_single_element(i)?;

        let el = vec![SolElement {
            name: "object".to_string(),
            value,
        }];

        Ok((i, el))
    }

    pub fn parse_managed_object_proxy<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        parse_object_proxy(i, amf3)
    }

    pub fn parse_serialization_proxy<'a>(
        i: &'a [u8],
        amf3: &AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<SolElement>> {
        let (i, value) = amf3.parse_single_element(i)?;

        let el = vec![SolElement {
            name: "proxy".to_string(),
            value,
        }];

        Ok((i, el))
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

pub mod encode {
    use crate::amf3::encoder::AMF3Encoder;
    use crate::amf3::CustomEncoder;
    use crate::types::{ClassDefinition, SolElement};
    use cookie_factory::bytes::be_u16;
    use cookie_factory::{gen, SerializeFn};
    use std::io::Write;

    pub struct ArrayCollection;

    impl CustomEncoder for ArrayCollection {
        fn encode<'a, 'b: 'a>(
            &self,
            elements: &'b [SolElement],
            class_def: &Option<ClassDefinition>,
            encoder: &AMF3Encoder,
        ) -> Vec<u8> {
            let v = Vec::new();
            let (bytes, size) = gen(self.do_encode(elements, class_def, encoder), v).unwrap();
            bytes
        }
    }

    impl ArrayCollection {
        fn do_encode<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            elements: &'b [SolElement],
            class_def: &'a Option<ClassDefinition>,
            encoder: &'a AMF3Encoder,
        ) -> impl SerializeFn<W> + 'a {
            let data = elements.get(0).unwrap();
            encoder.write_value(&data.value)
        }
    }

    pub struct ObjectProxy;

    impl CustomEncoder for ObjectProxy {
        fn encode<'a, 'b: 'a>(
            &self,
            elements: &'b [SolElement],
            class_def: &Option<ClassDefinition>,
            encoder: &AMF3Encoder,
        ) -> Vec<u8> {
            let v = Vec::new();
            let (bytes, size) = gen(self.do_encode(elements, class_def, encoder), v).unwrap();
            bytes
        }
    }

    impl ObjectProxy {
        fn do_encode<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            elements: &'b [SolElement],
            class_def: &'a Option<ClassDefinition>,
            encoder: &'a AMF3Encoder,
        ) -> impl SerializeFn<W> + 'a {
            let data = elements.get(0).unwrap();
            encoder.write_value(&data.value)
        }
    }

    pub fn register_encoders(encoder: &mut AMF3Encoder) {
        encoder.external_encoders.insert(
            "flex.messaging.io.ArrayCollection".to_string(),
            Box::new(ArrayCollection {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.ObjectProxy".to_string(),
            Box::new(ObjectProxy {}),
        );
    }
}
