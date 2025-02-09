#[derive(Debug)]
pub(crate) enum Ast<'a> {
    SimpleString(&'a [u8]),
}
