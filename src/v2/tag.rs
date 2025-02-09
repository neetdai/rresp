#[derive(Debug, PartialEq)]
pub(crate) enum Tag<'a> {
    SimpleString(&'a [u8]),
    SimpleError(&'a [u8]),
    BlobString(&'a [u8]),
    Null,
    Integer(i64),
    Array(usize),
}
