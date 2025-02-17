use crate::{common::Encoder, Error};

pub fn encode<'a, E>(frame: E::Frame<'a>) -> Result<E::Item, Error>
where
    E: Encoder,
{
    E::encode(frame)
}
