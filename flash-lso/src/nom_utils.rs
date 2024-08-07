use crate::errors::Error;
use nom::bytes::complete::take;
use nom::combinator::map_res;

use crate::write::WriteExt;
use nom::IResult;
use std::io::Write;

pub(crate) type AMFResult<'a, T> = IResult<&'a [u8], T, Error<'a>>;

pub(crate) fn write_string<'a, 'b: 'a, W: Write + 'a>(
    writer: &mut W,
    s: &'b str,
) -> std::io::Result<()> {
    writer.write_u16(s.len() as u16)?;
    writer.write_all(s.as_bytes())?;
    Ok(())
}

pub(crate) fn take_str(i: &[u8], length: u16) -> AMFResult<'_, &str> {
    map_res(take(length), std::str::from_utf8)(i)
}
