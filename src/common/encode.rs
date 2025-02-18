use super::Error;

pub trait Encoder {
    type Frame<'a>: EncodeLen;
    type Item;

    fn encode(frame: Self::Frame<'_>) -> Result<Self::Item, Error>;
}

pub trait EncodeLen {
    fn encode_len(&self) -> usize;
}
