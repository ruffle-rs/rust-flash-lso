mod type_marker;

pub mod decoder {
    use crate::amf0::type_marker::TypeMarker;
    use crate::types::{ClassDefinition, SolElement, Value};
    use crate::{amf3, PADDING};
    use nom::bytes::complete::tag;
    use nom::combinator::map;
    use nom::error::{make_error, ErrorKind};
    use nom::multi::{many0, many_m_n};
    use nom::number::complete::{be_f64, be_u16, be_u32, be_u8};
    use nom::take_str;
    use nom::Err;
    use nom::IResult;
    use std::convert::{TryFrom, TryInto};
    use std::rc::Rc;

    pub(crate) fn parse_string(i: &[u8]) -> IResult<&[u8], &str> {
        let (i, length) = be_u16(i)?;
        take_str!(i, length)
    }

    fn parse_element_number(i: &[u8]) -> IResult<&[u8], Value> {
        map(be_f64, Value::Number)(i)
    }

    fn parse_element_bool(i: &[u8]) -> IResult<&[u8], Value> {
        map(be_u8, |num: u8| Value::Bool(num > 0))(i)
    }

    fn parse_element_string(i: &[u8]) -> IResult<&[u8], Value> {
        map(parse_string, |s: &str| Value::String(s.to_string()))(i)
    }

    fn parse_element_object(i: &[u8]) -> IResult<&[u8], Value> {
        map(parse_array_element, |elms: Vec<SolElement>| {
            Value::Object(elms, None)
        })(i)
    }

    fn parse_element_movie_clip(i: &[u8]) -> IResult<&[u8], Value> {
        // Reserved but unsupported
        Err(Err::Error(make_error(i, ErrorKind::Tag)))
    }

    #[allow(clippy::let_and_return)]
    fn parse_element_mixed_array(i: &[u8]) -> IResult<&[u8], Value> {
        let (i, array_length) = be_u32(i)?;
        // this `let x = ...` fixes a borrow error on array_length
        let x = map(parse_array_element, |elms: Vec<SolElement>| {
            Value::ECMAArray(Vec::new(), elms, array_length)
        })(i);

        x
    }

    fn parse_element_reference(i: &[u8]) -> IResult<&[u8], Value> {
        // References arent supported
        Err(Err::Error(make_error(i, ErrorKind::Tag)))
    }

    fn parse_element_array(i: &[u8]) -> IResult<&[u8], Value> {
        let (i, length) = be_u32(i)?;

        let length_usize = length
            .try_into()
            .map_err(|_| Err::Error(make_error(i, ErrorKind::Digit)))?;

        // There must be at least `length_usize` bytes (u8) to read this, this prevents OOM errors with v.large arrays
        if i.len() < length_usize {
            return Err(Err::Error(make_error(i, ErrorKind::TooLarge)));
        }

        // This must parse length elements
        let (i, elements) = many_m_n(length_usize, length_usize, parse_single_element)(i)?;

        Ok((
            i,
            Value::StrictArray(elements.into_iter().map(Rc::new).collect()),
        ))
    }

    fn parse_element_date(i: &[u8]) -> IResult<&[u8], Value> {
        let (i, millis) = be_f64(i)?;
        let (i, time_zone) = be_u16(i)?;

        Ok((i, Value::Date(millis, Some(time_zone))))
    }

    fn parse_element_long_string(i: &[u8]) -> IResult<&[u8], Value> {
        let (i, length) = be_u32(i)?;
        let (i, str) = take_str!(i, length)?;

        Ok((i, Value::String(str.to_string())))
    }

    fn parse_element_record_set(i: &[u8]) -> IResult<&[u8], Value> {
        // Unsupported
        Err(Err::Error(make_error(i, ErrorKind::Tag)))
    }

    fn parse_element_xml(i: &[u8]) -> IResult<&[u8], Value> {
        let (i, content) = parse_element_long_string(i)?;
        if let Value::String(content_string) = content {
            Ok((i, Value::XML(content_string, true)))
        } else {
            // Will never happen
            Err(Err::Error(make_error(i, ErrorKind::Digit)))
        }
    }

    #[allow(clippy::let_and_return)]
    fn parse_element_typed_object(i: &[u8]) -> IResult<&[u8], Value> {
        let (i, name) = parse_string(i)?;

        let x = map(parse_array_element, |elms: Vec<SolElement>| {
            Value::Object(
                elms,
                Some(ClassDefinition::default_with_name(name.to_string())),
            )
        })(i);
        x
    }

    fn parse_element_amf3(i: &[u8]) -> IResult<&[u8], Value> {
        // Hopefully amf3 objects wont have references
        let (i, x) = amf3::AMF3Decoder::default().parse_element_object(i)?;
        Ok((i, Value::AMF3(x)))
    }

    fn read_type_marker(i: &[u8]) -> IResult<&[u8], TypeMarker> {
        let (i, type_) = be_u8(i)?;
        Ok((
            i,
            TypeMarker::try_from(type_).unwrap_or(TypeMarker::Unsupported),
        ))
    }

    fn parse_single_element(i: &[u8]) -> IResult<&[u8], Value> {
        let (i, type_) = read_type_marker(i)?;

        match type_ {
            TypeMarker::Number => parse_element_number(i),
            TypeMarker::Boolean => parse_element_bool(i),
            TypeMarker::String => parse_element_string(i),
            TypeMarker::Object => parse_element_object(i),
            TypeMarker::MovieClip => parse_element_movie_clip(i),
            TypeMarker::Null => Ok((i, Value::Null)),
            TypeMarker::Undefined => Ok((i, Value::Undefined)),
            TypeMarker::Reference => parse_element_reference(i),
            TypeMarker::MixedArrayStart => parse_element_mixed_array(i),
            TypeMarker::Array => parse_element_array(i),
            TypeMarker::Date => parse_element_date(i),
            TypeMarker::LongString => parse_element_long_string(i),
            TypeMarker::Unsupported => Ok((i, Value::Unsupported)),
            TypeMarker::RecordSet => parse_element_record_set(i),
            TypeMarker::XML => parse_element_xml(i),
            TypeMarker::TypedObject => parse_element_typed_object(i),
            TypeMarker::AMF3 => parse_element_amf3(i),
            TypeMarker::ObjectEnd => Err(Err::Error(make_error(i, ErrorKind::Digit))),
        }
    }

    fn parse_element(i: &[u8]) -> IResult<&[u8], SolElement> {
        let (i, name) = parse_string(i)?;

        map(parse_single_element, move |v| SolElement {
            name: name.to_string(),
            value: Rc::new(v),
        })(i)
    }

    fn parse_element_and_padding(i: &[u8]) -> IResult<&[u8], SolElement> {
        let (i, e) = parse_element(i)?;
        let (i, _) = tag(PADDING)(i)?;

        Ok((i, e))
    }

    //TODO: can this be done better somehow??
    fn parse_array_element(i: &[u8]) -> IResult<&[u8], Vec<SolElement>> {
        let mut out = Vec::new();

        let mut i = i;
        loop {
            let (k, _) = parse_string(i)?;
            let (k, next_type) = read_type_marker(k)?;
            if next_type == TypeMarker::ObjectEnd {
                i = k;
                break;
            }

            let (j, e) = parse_element(i)?;
            i = j;

            out.push(e.clone());
        }

        Ok((i, out))
    }

    pub fn parse_body(i: &[u8]) -> IResult<&[u8], Vec<SolElement>> {
        many0(parse_element_and_padding)(i)
    }
}

pub mod encoder {
    use crate::types::{SolElement, Value};
    use crate::PADDING;
    use cookie_factory::bytes::{be_f64, be_u16, be_u32, be_u8};
    use cookie_factory::{SerializeFn, WriteContext};
    use std::io::Write;

    use crate::amf0::type_marker::TypeMarker;
    use crate::amf3::encoder::AMF3Encoder;
    use crate::encoder::write_string;
    use cookie_factory::combinator::slice;
    use cookie_factory::combinator::string;
    use cookie_factory::multi::all;
    use cookie_factory::sequence::tuple;
    use std::ops::Deref;
    use std::rc::Rc;

    fn write_type_marker<'a, 'b: 'a, W: Write + 'a>(type_: TypeMarker) -> impl SerializeFn<W> + 'a {
        be_u8(type_ as u8)
    }

    fn write_number_element<'a, 'b: 'a, W: Write + 'a>(s: f64) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::Number), be_f64(s)))
    }

    fn write_bool_element<'a, 'b: 'a, W: Write + 'a>(s: bool) -> impl SerializeFn<W> + 'a {
        tuple((
            write_type_marker(TypeMarker::Boolean),
            be_u8(if s { 1u8 } else { 0u8 }),
        ))
    }

    fn write_long_string_content<'a, 'b: 'a, W: Write + 'a>(
        s: &'b str,
    ) -> impl SerializeFn<W> + 'a {
        tuple((be_u32(s.len() as u32), string(s)))
    }

    fn write_long_string_element<'a, 'b: 'a, W: Write + 'a>(
        s: &'b str,
    ) -> impl SerializeFn<W> + 'a {
        tuple((
            write_type_marker(TypeMarker::LongString),
            write_long_string_content(s),
        ))
    }

    fn write_string_element<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((write_type_marker(TypeMarker::String), write_string(s)))
    }

    fn write_object_element<'a, 'b: 'a, W: Write + 'a>(
        o: &'b [SolElement],
    ) -> impl SerializeFn<W> + 'a {
        tuple((
            write_type_marker(TypeMarker::Object),
            all(o.iter().map(write_element)),
            be_u16(0),
            write_type_marker(TypeMarker::ObjectEnd),
        ))
    }

    fn write_null_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        write_type_marker(TypeMarker::Null)
    }

    fn write_undefined_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        write_type_marker(TypeMarker::Undefined)
    }

    fn write_strict_array_element<'a, 'b: 'a, W: Write + 'a>(
        elements: &'b [Rc<Value>],
    ) -> impl SerializeFn<W> + 'a {
        tuple((
            write_type_marker(TypeMarker::Array),
            be_u32(elements.len() as u32),
            all(elements.iter().map(write_value)),
        ))
    }

    fn write_date_element<'a, 'b: 'a, W: Write + 'a>(
        date: f64,
        tz: Option<u16>,
    ) -> impl SerializeFn<W> + 'a {
        tuple((
            write_type_marker(TypeMarker::Date),
            be_f64(date),
            be_u16(tz.unwrap_or(0)),
        ))
    }

    fn write_unsupported_element<'a, 'b: 'a, W: Write + 'a>() -> impl SerializeFn<W> + 'a {
        write_type_marker(TypeMarker::Unsupported)
    }

    fn write_xml_element<'a, 'b: 'a, W: Write + 'a>(content: &'b str) -> impl SerializeFn<W> + 'a {
        tuple((
            write_type_marker(TypeMarker::XML),
            write_long_string_content(content),
        ))
    }

    fn write_typed_object_element<'a, 'b: 'a, W: Write + 'a>(
        name: &'b str,
        elements: &'b [SolElement],
    ) -> impl SerializeFn<W> + 'a {
        tuple((
            write_type_marker(TypeMarker::TypedObject),
            write_string(name),
            all(elements.iter().map(write_element)),
            be_u16(0),
            write_type_marker(TypeMarker::ObjectEnd),
        ))
    }

    fn write_mixed_array<'a, 'b: 'a, W: Write + 'a>(
        elements: &'b [SolElement],
        length: u32,
    ) -> impl SerializeFn<W> + 'a {
        //TODO: what is the u16 padding
        //TODO: sometimes array length is ignored (u32) sometimes its: elements.len() as u32

        tuple((
            write_type_marker(TypeMarker::MixedArrayStart),
            be_u32(length),
            all(elements.iter().map(write_element)),
            be_u16(0),
            write_type_marker(TypeMarker::ObjectEnd),
        ))
    }

    fn write_value<'a, 'b: 'a, W: Write + 'a>(element: &'b Rc<Value>) -> impl SerializeFn<W> + 'a {
        move |out: WriteContext<W>| match element.deref() {
            Value::Number(n) => write_number_element(*n)(out),
            Value::Bool(b) => write_bool_element(*b)(out),
            Value::String(s) => {
                if s.len() > 65535 {
                    write_long_string_element(s)(out)
                } else {
                    write_string_element(s)(out)
                }
            }
            Value::Object(elements, class_def) => {
                if let Some(class_def) = class_def {
                    write_typed_object_element(&class_def.name, elements)(out)
                } else {
                    write_object_element(elements)(out)
                }
            }
            Value::Null => write_null_element()(out),
            Value::Undefined => write_undefined_element()(out),
            Value::StrictArray(a) => write_strict_array_element(a)(out),
            Value::Date(d, tz) => write_date_element(*d, *tz)(out),
            Value::Unsupported => write_unsupported_element()(out),
            Value::XML(x, _string) => write_xml_element(x)(out),
            Value::ECMAArray(_dense, elems, elems_length) => {
                write_mixed_array(elems, *elems_length)(out)
            }
            Value::AMF3(e) => AMF3Encoder::default().write_value_element(e)(out),
            _ => {
                write_unsupported_element()(out) /* Not in amf0, TODO: use the amf3 embedding for every thing else */
            }
        }
    }

    fn write_element<'a, 'b: 'a, W: Write + 'a>(
        element: &'b SolElement,
    ) -> impl SerializeFn<W> + 'a {
        tuple((write_string(&element.name), write_value(&element.value)))
    }

    fn write_element_and_padding<'a, 'b: 'a, W: Write + 'a>(
        element: &'b SolElement,
    ) -> impl SerializeFn<W> + 'a {
        tuple((write_element(element), slice(PADDING)))
    }

    pub fn write_body<'a, 'b: 'a, W: Write + 'a>(
        elements: &'b [SolElement],
    ) -> impl SerializeFn<W> + 'a {
        all(elements.iter().map(write_element_and_padding))
    }
}
