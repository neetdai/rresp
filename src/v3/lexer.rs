use std::iter::Peekable;

use lexical::{format::STANDARD, parse_with_options, ParseIntegerOptions};
use memchr::Memchr;

use crate::Error;

// use super::tag::{Tag, TagType};
use super::tag::{ArrayTag, BasicTag, BasicTagType, ToFrame};

type ScanResult<T> = Result<T, Error>;

#[derive(Debug)]
enum Status {
    None,
    Array(Vec<Box<dyn ToFrame>>, usize),
    Set(Vec<Box<dyn ToFrame>>, usize),
    Map(Vec<Box<dyn ToFrame>>, usize),
}

#[derive(Debug)]
pub(crate) struct Lexer<'a> {
    input: &'a [u8],
    scanner: Peekable<Memchr<'a>>,
    last_position: usize,
    status: Status,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        let scanner = Memchr::new(b'\r', input);
        Self {
            input,
            scanner: scanner.peekable(),
            last_position: 0,
            status: Status::None,
        }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.last_position
    }

    fn walk(&mut self) -> Option<usize> {
        let end_position = loop {
            let end_position = self.scanner.next()?;
            if let Some(b'\n') = self.input.get(end_position + 1) {
                break end_position;
            }
        };
        Some(end_position)
    }

    fn match_tag(
        &mut self,
        start_position: usize,
        mut end_position: usize,
    ) -> Option<ScanResult<Box<dyn ToFrame>>> {
        let first = self.input.get(start_position)?;
        let mut start_position = start_position + 1;

        match first {
            b'+' => {
                self.last_position = end_position + 2;
                // TagType::SimpleString
                Some(Ok(Box::new(BasicTag {tag_type: BasicTagType::SimpleString, start_position, end_position })))
            }
            b'-' => {
                self.last_position = end_position + 2;
                // TagType::SimpleError
                Some(Ok(Box::new(BasicTag {tag_type: BasicTagType::SimpleError, start_position, end_position })))
            }
            b':' => {
                self.last_position = end_position + 2;
                // TagType::Integer
                Some(Ok(Box::new(BasicTag {tag_type: BasicTagType::Integer, start_position, end_position })))
            }
            b'|' => {
                self.last_position = end_position + 2;
                // TagType::Attribute
                todo!()
            }
            b'$' => {
                let follow = self.input.get(start_position..end_position)?;
                let options = ParseIntegerOptions::new();
                let len_result = parse_with_options::<isize, _, STANDARD>(follow, &options);
                match len_result {
                    Ok(-1) => Some(Ok(Box::new( BasicTag {tag_type: BasicTagType::Null, start_position, end_position}))),
                    Ok(len) => {
                        start_position = end_position + 2;
                        end_position = self.scanner.next()?;
                        let len = len as usize;
                        if len > end_position - start_position {
                            return Some(Err(Error::from(Error::InvalidBulkString)));
                        }

                        self.last_position = end_position + 2;
                        // TagType::BulkString
                        Some(Ok(Box::new(BasicTag {tag_type: BasicTagType::BulkString, start_position, end_position})))
                    }
                    Err(e) => return Some(Err(Error::from(e))),
                }
            }
            b'*' => {
                let follow = self.input.get(start_position..end_position)?;
                let options = ParseIntegerOptions::new();
                let len_result = parse_with_options::<isize, _, STANDARD>(follow, &options);
                self.last_position = end_position + 2;
                match len_result {
                    Ok(-1) => Some(Ok(Box::new( BasicTag {tag_type: BasicTagType::Null, start_position, end_position}))),
                    Ok(len) if len < 0 => return Some(Err(Error::from(Error::InvalidArray))),
                    // Ok(_) => TagType::Array,
                    Ok(_) => todo!(),
                    Err(e) => return Some(Err(Error::from(e))),
                }
            }
            b'_' => {
                self.last_position = end_position + 2;
                // TagType::Null
                Some(Ok(Box::new( BasicTag {tag_type: BasicTagType::Null, start_position, end_position})))
            }
            b'#' => {
                self.last_position = end_position + 2;
                // TagType::Boolean
                Some(Ok(Box::new( BasicTag {tag_type: BasicTagType::Boolean, start_position, end_position})))
            }
            b',' => {
                self.last_position = end_position + 2;
                // TagType::Double
                Some(Ok(Box::new( BasicTag {tag_type: BasicTagType::Double, start_position, end_position})))
            }
            b'(' => {
                self.last_position = end_position + 2;
                // TagType::BigNumber
                Some(Ok(Box::new( BasicTag {tag_type: BasicTagType::BigNumber, start_position, end_position})))
            }
            b'!' => {
                let follow = self.input.get(start_position..end_position)?;
                let options = ParseIntegerOptions::new();
                let len_result = parse_with_options::<usize, _, STANDARD>(follow, &options);
                match len_result {
                    Ok(len) => {
                        start_position = end_position + 2;
                        end_position = self.walk()?;
                        if len > end_position - start_position {
                            return Some(Err(Error::InvalidError));
                        } else {
                            self.last_position = end_position + 2;
                            // TagType::BulkError
                            Some(Ok(Box::new(BasicTag {tag_type: BasicTagType::BulkError, start_position, end_position})))
                        }
                    }
                    Err(e) => return Some(Err(Error::from(e))),
                }
            }
            b'~' => {
                self.last_position = end_position + 2;
                TagType::Set
            }
            b'%' => {
                self.last_position = end_position + 2;
                TagType::Map
            }
            b'=' => {
                let follow = self.input.get(start_position..end_position)?;
                let options = ParseIntegerOptions::new();
                let len_result = parse_with_options::<usize, _, STANDARD>(follow, &options);
                match len_result {
                    Ok(len) => {
                        start_position = end_position + 2;
                        end_position = self.walk()?;
                        if len > end_position - start_position {
                            return Some(Err(Error::InvalidError));
                        } else {
                            self.last_position = end_position + 2;
                            // TagType::VerbatimString
                            Some(Ok(Box::new(BasicTag { tag_type: BasicTagType::VerbatimString, start_position, end_position})))
                        }
                    }
                    Err(e) => return Some(Err(Error::from(e))),
                }
            }
            _ => return Some(Err(Error::Unknown)),
        }

        // Some(Ok(Tag {
        //     tag_type,
        //     start_position,
        //     end_position,
        // }))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = ScanResult<Box<dyn ToFrame>>;

    fn next(&mut self) -> Option<Self::Item> {
        let end_position = self.walk()?;
        self.match_tag(self.last_position, end_position)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.last_position, Some(self.input.len()))
    }
}

mod test {
    use super::*;

    #[test]
    fn test_simple_string() {
        let input = b"+hello\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Tag {
                tag_type: TagType::SimpleString,
                start_position: 1,
                end_position: 6
            }
        );
    }

    #[test]
    fn test_simple_error() {
        let input = b"-err\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Tag {
                tag_type: TagType::SimpleError,
                start_position: 1,
                end_position: 4
            }
        );
    }

    #[test]
    fn test_bulk_string() {
        let input = b"$5\r\nhello\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Tag {
                tag_type: TagType::BulkString,
                start_position: 4,
                end_position: 9
            }
        );
    }

    #[test]
    fn test_integer() {
        let input = b":1\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Tag {
                tag_type: TagType::Integer,
                start_position: 1,
                end_position: 2
            }
        );

        let input = b":-1\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Tag {
                tag_type: TagType::Integer,
                start_position: 1,
                end_position: 3
            }
        );

        let input = b":+1\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Tag {
                tag_type: TagType::Integer,
                start_position: 1,
                end_position: 3
            }
        );
    }

    #[test]
    fn test_boolean() {
        let input = b"#t\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Tag {
                tag_type: TagType::Boolean,
                start_position: 1,
                end_position: 2
            }
        );
    }

    #[test]
    fn test_big_number() {
        let input = b"(0123456789\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Tag {
                tag_type: TagType::BigNumber,
                start_position: 1,
                end_position: 11
            }
        );
    }

    #[test]
    fn test_bulk_error() {
        let input = b"!5\r\nerror\r\n";
        let mut lexer = Lexer::new(input);

        assert_eq!(
            lexer.next().unwrap().unwrap(),
            Tag {
                tag_type: TagType::BulkError,
                start_position: 4,
                end_position: 9
            }
        );
    }
}
