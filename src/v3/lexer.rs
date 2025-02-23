use lexical::{parse_with_options, ParseIntegerOptions, format::STANDARD};
use memchr::Memchr;

use crate::Error;

use super::tag::{Tag, TagType};

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

    fn walk(&mut self) -> Option<usize> {
        let end_position = loop {
            let end_position = self.scanner.next()?;
            if let Some(b'\n') = self.input.get(end_position + 1) {
                break end_position;
            }
        };
        Some(end_position)

    }

    fn match_tag(&mut self, start_position: usize, mut end_position: usize) -> Option<ScanResult<Tag>> {
        let first = self.input.get(start_position)?;
        let mut start_position = start_position + 1;

        let tag_type = match first {
            b'+' => TagType::SimpleString,
            b'-' => TagType::SimpleError,
            b':' => TagType::Integer,
            b'$' => {
                let follow = self.input.get(start_position..end_position)?;
                let options = ParseIntegerOptions::new();
                let len_result = parse_with_options::<isize, _, STANDARD>(follow, &options);
                match len_result {
                    Ok(-1) => TagType::Null,
                    Ok(len) => {
                        start_position = end_position + 2;
                        end_position = self.scanner.next()?;
                        self.last_position = start_position;
                        TagType::BulkString
                    },
                    Err(e) => return Some(Err(Error::from(e))),
                }
            },
            b'*' => {
                TagType::Array
            }
            _ => {
                todo!()
            }
        };

        Some(Ok(Tag {
            tag_type,
            start_position,
            end_position,
        }))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = ScanResult<Tag>;

    fn next(&mut self) -> Option<Self::Item> {
        let end_position = self.walk()?;
        let tag_result = self.match_tag(self.last_position, end_position);
        self.last_position = end_position + 2;

        tag_result
    }
}