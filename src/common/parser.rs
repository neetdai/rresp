use super::error::Error;

pub trait Parser {
    type Frame;

    fn parse<T>(input: T) -> Result<Self::Frame, Error> where T: AsRef<[u8]>;
}