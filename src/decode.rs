use crate::common::{Error, Parser};


pub fn decode<'a, D>(input: &'a [u8]) -> Result<Option<D::Frame<'a>>, Error> where D: Parser {
    D::parse(input)
}