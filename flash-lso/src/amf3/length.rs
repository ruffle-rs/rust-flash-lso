use crate::amf3::write::AMF3Encoder;
use cookie_factory::SerializeFn;
use std::io::Write;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) enum Length {
    Size(u32),
    Reference(usize),
}

impl Length {
    pub(crate) fn is_reference(&self) -> bool {
        matches!(self, Length::Reference(_))
    }

    pub(crate) fn is_size(&self) -> bool {
        matches!(self, Length::Size(_))
    }

    pub(crate) fn as_position(&self) -> Option<usize> {
        match self {
            Length::Reference(x) => Some(*x),
            _ => None,
        }
    }

    pub(crate) fn write<'a, 'b: 'a, W: Write + 'a>(
        &self,
        amf3: &AMF3Encoder,
    ) -> impl SerializeFn<W> + 'a {
        write_length(amf3, self)
    }
}

fn write_length<'a, 'b: 'a, W: Write + 'a>(
    amf3: &AMF3Encoder,
    s: &Length,
) -> impl SerializeFn<W> + 'a {
    match s {
        Length::Size(x) => {
            // With the last bit set
            amf3.write_int(((x << 1) | 0b1) as i32)
        }
        Length::Reference(x) => amf3.write_int((x << 1) as i32),
    }
}
