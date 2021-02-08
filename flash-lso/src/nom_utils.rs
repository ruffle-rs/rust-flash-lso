use crate::errors::Error;
use cookie_factory::bytes::be_u16;
use cookie_factory::combinator::string;
use cookie_factory::sequence::tuple;
use cookie_factory::SerializeFn;
use nom::bytes::complete::take;
use nom::combinator::map_res;

use nom::IResult;
use std::io::Write;

pub(crate) type AMFResult<'a, T> = IResult<&'a [u8], T, Error<'a>>;

pub(crate) fn either<Fa, Fb, W: Write>(b: bool, t: Fa, f: Fb) -> impl SerializeFn<W>
where
    Fa: SerializeFn<W>,
    Fb: SerializeFn<W>,
{
    move |out| {
        if b {
            t(out)
        } else {
            f(out)
        }
    }
}

pub(crate) fn write_string<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
    tuple((be_u16(s.len() as u16), string(s)))
}

pub(crate) fn take_str(i: &[u8], length: u16) -> AMFResult<'_, &str> {
    map_res(take(length), std::str::from_utf8)(i)
}
