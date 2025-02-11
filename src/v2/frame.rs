use super::Lexer;

#[derive(Debug)]
pub enum Frame<'a> {
    BlobString(&'a [u8]),
    Null,
    Integer(i64),
    Array(Vec<Frame<'a>>),
    SimpleError(&'a [u8]),
    SimpleString(&'a [u8]),
}
