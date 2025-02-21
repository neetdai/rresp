
#[derive(Debug)]
pub(crate) enum Tag<'a> {
    BulkString(&'a [u8]),
}