pub mod decode {
    use nom::IResult;

    pub fn parse_flex<'a>(i: &'a [u8]) -> IResult<&'a [u8], ()> {
        Ok((i, ()))
    }
}
