use memchr::Memchr;

use crate::Error;

use super::tag::Tag;

type ScanResult<T> = Result<T, Error>;

#[derive(Debug)]
pub(crate) struct Lexer<'a> {
    input: &'a [u8],
    scanner: Memchr<'a>,
    last_position: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        let scanner = Memchr::new(b'\r', input);
        Self {
            input,
            scanner,
            last_position: 0,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = ScanResult<Tag<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}