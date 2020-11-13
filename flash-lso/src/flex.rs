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

pub mod decode {
    use crate::amf3::AMF3Decoder;
    use crate::flex::{
        BODY_FLAG, CLIENT_ID_BYTES_FLAG, CLIENT_ID_FLAG, CORRELATION_ID_BYTES_FLAG,
        CORRELATION_ID_FLAG, DESTINATION_ID_FLAG, HEADERS_FLAG, MESSAGE_ID_BYTES_FLAG,
        MESSAGE_ID_FLAG, NEXT_FLAG, OPERATION_FLAG, TIMESTAMP_FLAG, TTL_FLAG,
    };
    use crate::types::Element;
    use nom::number::complete::be_u8;
    use nom::IResult;
    use std::rc::Rc;

    fn parse_abstract_message_flags(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
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

    fn parse_abstract_message<'a>(
        i: &'a [u8],
        amf3: &mut AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<Element>> {
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

    fn parse_async_message<'a>(
        i: &'a [u8],
        amf3: &mut AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<Element>> {
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

    fn parse_acknowledge_message<'a>(
        i: &'a [u8],
        amf3: &mut AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<Element>> {
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
                            name: format!("children_acknowledge_{}", j),
                            value,
                        });
                        k = jj;
                    }
                }
            }
        }

        Ok((k, elements))
    }

    fn parse_command_message<'a>(
        i: &'a [u8],
        amf3: &mut AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<Element>> {
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

    // all arrays
    fn parse_array_collection<'a>(
        i: &'a [u8],
        amf3: &mut AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<Element>> {
        let (i, value) = amf3.parse_single_element(i)?;

        let el = vec![Element {
            name: "data".to_string(),
            value,
        }];

        Ok((i, el))
    }

    // all proxies
    fn parse_object_proxy<'a>(
        i: &'a [u8],
        amf3: &mut AMF3Decoder,
    ) -> IResult<&'a [u8], Vec<Element>> {
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
        decoder.external_decoders.insert(
            "flex.messaging.io.AbstractMessage".to_string(),
            Rc::new(Box::new(parse_abstract_message)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.AsyncMessage".to_string(),
            Rc::new(Box::new(parse_async_message)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.AsyncMessageExt".to_string(),
            Rc::new(Box::new(parse_async_message)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.AcknowledgeMessage".to_string(),
            Rc::new(Box::new(parse_acknowledge_message)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.AcknowledgeMessageExt".to_string(),
            Rc::new(Box::new(parse_acknowledge_message)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.CommandMessage".to_string(),
            Rc::new(Box::new(parse_command_message)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.CommandMessageExt".to_string(),
            Rc::new(Box::new(parse_command_message)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.ErrorMessage".to_string(),
            Rc::new(Box::new(parse_acknowledge_message)),
        );

        decoder.external_decoders.insert(
            "flex.messaging.io.ArrayCollection".to_string(),
            Rc::new(Box::new(parse_array_collection)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.ArrayList".to_string(),
            Rc::new(Box::new(parse_array_collection)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.ObjectProxy".to_string(),
            Rc::new(Box::new(parse_object_proxy)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.ManagedObjectProxy".to_string(),
            Rc::new(Box::new(parse_object_proxy)),
        );
        decoder.external_decoders.insert(
            "flex.messaging.io.SerializationProxy".to_string(),
            Rc::new(Box::new(parse_object_proxy)),
        );
    }
}

pub mod encode {
    use crate::amf3::encoder::AMF3Encoder;
    use crate::amf3::CustomEncoder;
    use crate::flex::{
        BODY_FLAG, CLIENT_ID_BYTES_FLAG, CLIENT_ID_FLAG, CORRELATION_ID_BYTES_FLAG,
        CORRELATION_ID_FLAG, DESTINATION_ID_FLAG, HEADERS_FLAG, MESSAGE_ID_BYTES_FLAG,
        MESSAGE_ID_FLAG, NEXT_FLAG, OPERATION_FLAG, TIMESTAMP_FLAG, TTL_FLAG,
    };
    use crate::types::{ClassDefinition, Element};
    use cookie_factory::bytes::be_u8;
    use cookie_factory::multi::all;
    use cookie_factory::sequence::tuple;
    use cookie_factory::{gen, SerializeFn};
    use std::io::Write;

    struct ArrayCollection;

    impl CustomEncoder for ArrayCollection {
        fn encode<'a>(
            &self,
            elements: &'a [Element],
            _class_def: &Option<ClassDefinition>,
            encoder: &AMF3Encoder,
        ) -> Vec<u8> {
            let v = Vec::new();
            let (bytes, _size) = gen(self.do_encode(elements, encoder), v).unwrap();
            bytes
        }
    }

    impl ArrayCollection {
        fn do_encode<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            elements: &'b [Element],
            encoder: &'a AMF3Encoder,
        ) -> impl SerializeFn<W> + 'a {
            let data = elements.get(0).unwrap();
            encoder.write_value_element(&data.value)
        }
    }

    struct ObjectProxy;

    impl CustomEncoder for ObjectProxy {
        fn encode<'a>(
            &self,
            elements: &'a [Element],
            _class_def: &Option<ClassDefinition>,
            encoder: &AMF3Encoder,
        ) -> Vec<u8> {
            let v = Vec::new();
            let (bytes, _size) = gen(self.do_encode(elements, encoder), v).unwrap();
            bytes
        }
    }

    impl ObjectProxy {
        fn do_encode<'a, 'b: 'a, W: Write + 'a>(
            &'a self,
            elements: &'b [Element],
            encoder: &'a AMF3Encoder,
        ) -> impl SerializeFn<W> + 'a {
            let data = elements.get(0).unwrap();
            encoder.write_value_element(&data.value)
        }
    }

    fn write_flags<'a, 'b: 'a, W: Write + 'a>(flags: &'a [u8]) -> impl SerializeFn<W> + 'a {
        all(flags.iter().enumerate().map(move |(index, flag)| {
            if index == flags.len() {
                be_u8(*flag & !NEXT_FLAG)
            } else {
                be_u8(*flag | NEXT_FLAG)
            }
        }))
    }

    struct AbstractMessage;

    impl CustomEncoder for AbstractMessage {
        fn encode<'a>(
            &self,
            elements: &'a [Element],
            _class_def: &Option<ClassDefinition>,
            encoder: &AMF3Encoder,
        ) -> Vec<u8> {
            let v = Vec::new();
            let (bytes, _size) = gen(write_abstract_message(elements, encoder), v).unwrap();
            bytes
        }
    }

    fn write_abstract_message<'a, 'b: 'a, W: Write + 'a>(
        elements: &'b [Element],
        encoder: &'a AMF3Encoder,
    ) -> impl SerializeFn<W> + 'a {
        move |out| {
            let mut flags = Vec::new();
            let mut new_elements = Vec::new();
            {
                let mut flag = 0;

                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "body")
                    .map(|e| e.value.clone())
                {
                    flag |= BODY_FLAG;
                    new_elements.push(v);
                }
                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "client_id")
                    .map(|e| e.value.clone())
                {
                    flag |= CLIENT_ID_FLAG;
                    new_elements.push(v);
                }
                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "destination")
                    .map(|e| e.value.clone())
                {
                    flag |= DESTINATION_ID_FLAG;
                    new_elements.push(v);
                }
                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "headers")
                    .map(|e| e.value.clone())
                {
                    flag |= HEADERS_FLAG;
                    new_elements.push(v);
                }
                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "message_id")
                    .map(|e| e.value.clone())
                {
                    flag |= MESSAGE_ID_FLAG;
                    new_elements.push(v);
                }
                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "timestamp")
                    .map(|e| e.value.clone())
                {
                    flag |= TIMESTAMP_FLAG;
                    new_elements.push(v);
                }
                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "ttl")
                    .map(|e| e.value.clone())
                {
                    flag |= TTL_FLAG;
                    new_elements.push(v);
                }

                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "children_1")
                    .map(|e| e.value.clone())
                {
                    flag |= 0b0100_0000;
                    new_elements.push(v);
                }

                flags.push(flag);
            }
            {
                let mut flag = 0;

                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "client_id_bytes")
                    .map(|e| e.value.clone())
                {
                    flag |= CLIENT_ID_BYTES_FLAG;
                    new_elements.push(v);
                }
                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "message_id_bytes")
                    .map(|e| e.value.clone())
                {
                    flag |= MESSAGE_ID_BYTES_FLAG;
                    new_elements.push(v);
                }

                for n in 2..7 {
                    if let Some(v) = elements
                        .iter()
                        .find(|e| e.name == format!("children_{}", n))
                        .map(|e| e.value.clone())
                    {
                        flag |= 0b1 << n;
                        new_elements.push(v);
                    }
                }

                flags.push(flag);
            }

            {
                let mut n = 0;
                let mut base = 8;
                let mut flag = 0;
                loop {
                    if let Some(v) = elements
                        .iter()
                        .find(|e| e.name == format!("children_{}", n + base))
                        .map(|e| e.value.clone())
                    {
                        flag |= 0b1 << n;
                        new_elements.push(v);
                    } else {
                        if flag != 0 {
                            flags.push(flag);
                        }
                        break;
                    }

                    n += 1;
                    if n > 7 {
                        n = 0;
                        base += 7;
                        flags.push(flag);
                        flag = 0;
                    }
                }
            }

            let x = tuple((
                write_flags(&flags),
                all(new_elements
                    .iter()
                    .map(move |v| encoder.write_value_element(v))),
            ))(out);
            x
        }
    }

    struct AsyncMessage;

    impl CustomEncoder for AsyncMessage {
        fn encode<'a>(
            &self,
            elements: &'a [Element],
            _class_def: &Option<ClassDefinition>,
            encoder: &AMF3Encoder,
        ) -> Vec<u8> {
            let v = Vec::new();
            let (bytes, _size) = gen(write_async_message(elements, encoder), v).unwrap();
            bytes
        }
    }

    fn write_async_message<'a, 'b: 'a, W: Write + 'a>(
        elements: &'b [Element],
        encoder: &'a AMF3Encoder,
    ) -> impl SerializeFn<W> + 'a {
        move |out| {
            let mut flags = Vec::new();
            let mut new_elements = Vec::new();
            {
                let mut flag = 0;

                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "correlation_id")
                    .map(|e| e.value.clone())
                {
                    flag |= CORRELATION_ID_FLAG;
                    new_elements.push(v);
                }

                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "correlation_id_bytes")
                    .map(|e| e.value.clone())
                {
                    flag |= CORRELATION_ID_BYTES_FLAG;
                    new_elements.push(v);
                }

                for n in 2..7 {
                    if let Some(v) = elements
                        .iter()
                        .find(|e| e.name == format!("children_async_{}", n))
                        .map(|e| e.value.clone())
                    {
                        flag |= 0b1 << n;
                        new_elements.push(v);
                    }
                }

                flags.push(flag);
            }
            {
                let mut n = 0;
                let mut base = 7;
                let mut flag = 0;
                loop {
                    if let Some(v) = elements
                        .iter()
                        .find(|e| e.name == format!("children_async_{}", n + base))
                        .map(|e| e.value.clone())
                    {
                        flag |= 0b1 << n;
                        new_elements.push(v);
                    } else {
                        if flag != 0 {
                            flags.push(flag);
                        }
                        break;
                    }

                    n += 1;
                    if n > 7 {
                        n = 0;
                        base += 7;
                        flags.push(flag);
                        flag = 0;
                    }
                }
            }

            let x = tuple((
                write_abstract_message(elements, encoder),
                write_flags(&flags),
                all(new_elements
                    .iter()
                    .map(move |v| encoder.write_value_element(v))),
            ))(out);
            x
        }
    }

    struct AcknowledgeMessage;

    impl CustomEncoder for AcknowledgeMessage {
        fn encode<'a>(
            &self,
            elements: &'a [Element],
            _class_def: &Option<ClassDefinition>,
            encoder: &AMF3Encoder,
        ) -> Vec<u8> {
            let v = Vec::new();
            let (bytes, _size) = gen(write_acknowledge_message(elements, encoder), v).unwrap();
            bytes
        }
    }

    fn write_acknowledge_message<'a, 'b: 'a, W: Write + 'a>(
        elements: &'b [Element],
        encoder: &'a AMF3Encoder,
    ) -> impl SerializeFn<W> + 'a {
        move |out| {
            let mut flags = Vec::new();
            let mut new_elements = Vec::new();
            {
                let mut n = 0;
                let mut base = 0;
                let mut flag = 0;
                loop {
                    if let Some(v) = elements
                        .iter()
                        .find(|e| e.name == format!("children_acknowledge_{}", n + base))
                        .map(|e| e.value.clone())
                    {
                        flag |= 0b1 << n;
                        new_elements.push(v);
                    } else {
                        if flag != 0 {
                            flags.push(flag);
                        }
                        break;
                    }

                    n += 1;
                    if n > 7 {
                        n = 0;
                        base += 7;
                        flags.push(flag);
                        flag = 0;
                    }
                }
            }

            let x = tuple((
                write_async_message(elements, encoder),
                write_flags(&flags),
                all(new_elements
                    .iter()
                    .map(move |v| encoder.write_value_element(v))),
            ))(out);
            x
        }
    }

    struct CommandMessage;

    impl CustomEncoder for CommandMessage {
        fn encode<'a>(
            &self,
            elements: &'a [Element],
            _class_def: &Option<ClassDefinition>,
            encoder: &AMF3Encoder,
        ) -> Vec<u8> {
            let v = Vec::new();
            let (bytes, _size) = gen(write_command_message(elements, encoder), v).unwrap();
            bytes
        }
    }

    fn write_command_message<'a, 'b: 'a, W: Write + 'a>(
        elements: &'b [Element],
        encoder: &'a AMF3Encoder,
    ) -> impl SerializeFn<W> + 'a {
        move |out| {
            let mut flags = Vec::new();
            let mut new_elements = Vec::new();
            {
                let mut flag = 0;

                if let Some(v) = elements
                    .iter()
                    .find(|e| e.name == "operation")
                    .map(|e| e.value.clone())
                {
                    flag |= OPERATION_FLAG;
                    new_elements.push(v);
                }

                for n in 1..7 {
                    if let Some(v) = elements
                        .iter()
                        .find(|e| e.name == format!("children_command_{}", n))
                        .map(|e| e.value.clone())
                    {
                        flag |= 0b1 << n;
                        new_elements.push(v);
                    }
                }

                flags.push(flag);
            }
            {
                let mut n = 0;
                let mut base = 8;
                let mut flag = 0;
                loop {
                    if let Some(v) = elements
                        .iter()
                        .find(|e| e.name == format!("children_command_{}", n + base))
                        .map(|e| e.value.clone())
                    {
                        flag |= 0b1 << n;
                        new_elements.push(v);
                    } else {
                        if flag != 0 {
                            flags.push(flag);
                        }
                        break;
                    }

                    n += 1;
                    if n > 7 {
                        n = 0;
                        base += 7;
                        flags.push(flag);
                        flag = 0;
                    }
                }
            }

            let x = tuple((
                write_async_message(elements, encoder),
                write_flags(&flags),
                all(new_elements
                    .iter()
                    .map(move |v| encoder.write_value_element(v))),
            ))(out);
            x
        }
    }

    /// Register the flex encoders into the given AMF3Encoder
    #[inline]
    pub fn register_encoders(encoder: &mut AMF3Encoder) {
        encoder.external_encoders.insert(
            "flex.messaging.io.ArrayCollection".to_string(),
            Box::new(ArrayCollection {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.ArrayList".to_string(),
            Box::new(ArrayCollection {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.AbstractMessage".to_string(),
            Box::new(AbstractMessage {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.AsyncMessage".to_string(),
            Box::new(AsyncMessage {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.AsyncMessageExt".to_string(),
            Box::new(AsyncMessage {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.AcknowledgeMessage".to_string(),
            Box::new(AcknowledgeMessage {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.AcknowledgeMessageExt".to_string(),
            Box::new(AcknowledgeMessage {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.CommandMessage".to_string(),
            Box::new(CommandMessage {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.CommandMessageExt".to_string(),
            Box::new(CommandMessage {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.ErrorMessage".to_string(),
            Box::new(AcknowledgeMessage {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.ObjectProxy".to_string(),
            Box::new(ObjectProxy {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.ManagedObjectProxy".to_string(),
            Box::new(ObjectProxy {}),
        );

        encoder.external_encoders.insert(
            "flex.messaging.io.SerializationProxy".to_string(),
            Box::new(ObjectProxy {}),
        );
    }
}
