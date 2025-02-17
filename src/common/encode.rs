use super::Error;

pub trait Encoder {
    type Frame<'a>;
    type Item;

    fn encode(frame: Self::Frame<'_>) -> Result<Self::Item, Error>;
}
