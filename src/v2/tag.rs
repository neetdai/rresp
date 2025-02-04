
#[derive(Debug)]
pub(crate) enum Tag<'a> {
    SimpleString(&'a [u8]),
}