use super::error::Error;

pub trait Parser {
    type Frame<'a>;

    fn parse<'a>(input: &'a [u8]) -> Result<Option<Self::Frame<'a>>, Error>;
}

pub trait Remaining {
    fn remaining(&self) -> usize;
}

pub trait ParseIter {
    type Item<'a>;
    type Iter<'a>: Iterator<Item = Self::Item<'a>> + Remaining;

    fn parse_iter<'a>(input: &'a [u8]) -> Self::Iter<'a>;
}
