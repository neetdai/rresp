use crate::common::Error;
use lexical::{format::STANDARD, parse_with_options, FromLexicalWithOptions, ParseIntegerOptions};
use memchr::memmem::{FindIter, Finder, FinderBuilder, Prefilter};

use super::{ast::Ast, tag::Tag, utils::CRLF};

type ScanResult<T> = Result<T, Error>;

#[derive(Debug)]
pub(crate) struct Lexer<'a> {
    input: &'a [u8],
    scanner: FindIter<'a, 'static>,
    last_position: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        let mut builder = FinderBuilder::new();
        builder.prefilter(Prefilter::Auto);
        let finder = builder.build_forward(&CRLF);
        let scanner = finder.find_iter(input).into_owned();

        Self {
            input,
            scanner,
            last_position: 0,
        }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.last_position
    }

    fn scan_blob_string(&mut self, len: usize) -> Option<ScanResult<Tag<'a>>> {
        let content = self.walk()?;
        if content.len() != len {
            Some(Err(Error::InvalidBlobString))
        } else {
            Some(Ok(Tag::BlobString(content)))
        }
    }

    fn match_ast(&mut self, split: &'a [u8]) -> Option<ScanResult<Tag<'a>>> {
        let (first, follow) = split.split_first()?;
        match first {
            b'+' => Some(Ok(Tag::SimpleString(follow))),
            b'-' => Some(Ok(Tag::SimpleError(follow))),
            b'$' => {
                let options = ParseIntegerOptions::new();
                let len_result = parse_with_options::<isize, _, STANDARD>(follow, &options);
                match len_result {
                    Ok(-1) => Some(Ok(Tag::Null)),
                    Ok(len) => self.scan_blob_string(len as usize),
                    Err(e) => Some(Err(Error::from(e))),
                }
            }
            b':' => {
                let options = ParseIntegerOptions::new();
                let num_result = parse_with_options::<i64, _, STANDARD>(follow, &options);
                Some(
                    num_result
                        .map_err(|e| Error::from(e))
                        .map(|num| Tag::Integer(num)),
                )
            }
            b'*' => {
                let options = ParseIntegerOptions::new();
                let len_result = parse_with_options::<usize, _, STANDARD>(follow, &options);
                Some(
                    len_result
                        .map_err(|e| Error::from(e))
                        .map(|len| Tag::Array(len))
                )
            }
            _ => Some(Err(Error::Unknown)),
        }
    }

    fn walk(&mut self) -> Option<&'a [u8]> {
        let end_position = self.scanner.next()?;
        let split = self.input.get(self.last_position..end_position)?;
        self.last_position = end_position + 2; // +2 to skip the CRLF
        Some(split)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = ScanResult<Tag<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let split = self.walk()?;
        self.match_ast(split)
    }
}

mod test {
    use super::*;

    #[test]
    fn test_simple_string() {
        let input = b"+hello\r\n+world\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Ok(Tag::SimpleString(b"hello")));
        assert_eq!(lexer.next().unwrap(), Ok(Tag::SimpleString(b"world")));
    }

    #[test]
    fn test_unknown() {
        let input = b"hello\r\nworld\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Err(Error::Unknown));
        assert_eq!(lexer.next().unwrap(), Err(Error::Unknown));
    }

    #[test]
    fn test_error() {
        let input = b"-error1\r\n-error2\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Ok(Tag::SimpleError(b"error1")));
        assert_eq!(lexer.next().unwrap(), Ok(Tag::SimpleError(b"error2")));
    }

    #[test]
    fn test_blob_string() {
        let input = b"$5\r\nhello\r\n$5\r\nworld\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Ok(Tag::BlobString(b"hello")));
        assert_eq!(lexer.next().unwrap(), Ok(Tag::BlobString(b"world")));

        let input = b"$-1\r\n$5\r\nhello\r\n";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next().unwrap(), Ok(Tag::Null));
        assert_eq!(lexer.next().unwrap(), Ok(Tag::BlobString(b"hello")));

        let input = b"$3.0\r\n";
        let mut lexer = Lexer::new(input);
        assert_eq!(
            lexer.next().unwrap(),
            Err(Error::SyntaxLen(lexical::Error::InvalidDigit(1)))
        );
    }

    #[test]
    fn test_number() {
        let input = b":1\r\n:-1\r\n:+1\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Ok(Tag::Integer(1)));
        assert_eq!(lexer.next().unwrap(), Ok(Tag::Integer(-1)));
        assert_eq!(lexer.next().unwrap(), Ok(Tag::Integer(1)));
    }

    #[test]
    fn test_array() {
        let input = b"*1\r\n$5\r\nhello\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Ok(Tag::Array(1)));
        assert_eq!(lexer.next().unwrap(), Ok(Tag::BlobString(b"hello")));

        let input = b"*0\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap(), Ok(Tag::Array(0)));

    }
}
