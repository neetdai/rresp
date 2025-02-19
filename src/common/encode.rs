use std::io::{Result as IoResult, Write};

use super::Error;

pub trait Encoder {
    type Frame<'a>: EncodeLen;
    type Item;

    fn encode(frame: Self::Frame<'_>) -> Result<Self::Item, Error>;
}

pub trait EncodeLen {
    fn encode_len(&self) -> usize;
}

pub trait EncodeWithWriter {
    type Frame<'a>: EncodeLen;

    fn encode_with_writer<W>(frame: Self::Frame<'_>, writer: &mut W) -> IoResult<()>
    where
        W: Write;
}
