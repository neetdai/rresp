use super::error::Error;

pub trait Parser {
    type Frame<'a>;

    fn parse<'a>(input: &'a [u8]) -> Result<Option<Self::Frame<'a>>, Error>;
}
