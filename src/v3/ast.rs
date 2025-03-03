use std::collections::VecDeque;

use lexical::{format::STANDARD, parse_with_options, ParseFloatOptions, ParseIntegerOptions};

use crate::common::Error;

use super::{frame::Frame, lexer::Lexer, tag::TagType};

#[derive(Debug)]
pub(crate) struct Ast<'a> {
    input: &'a [u8],
    lexer: Lexer<'a>,
}

impl<'a> Ast<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            lexer: Lexer::new(input),
        }
    }

    fn next_frame(&mut self) -> Option<Result<Frame<'a>, Error>> {
        match self.lexer.next() {
            Some(Ok(tag)) => match tag.tag_type {
                TagType::Boolean => Some(self.parse_boolean(tag.start_position, tag.end_position)),
                TagType::SimpleString => Some(self.parse_simple_string(tag.start_position, tag.end_position)),
                TagType::SimpleError => Some(self.parse_simple_error(tag.start_position, tag.end_position)),
                TagType::Null => Some(Ok(Frame::Null { data: () })),
                TagType::Integer => Some(self.parse_integer(tag.start_position, tag.end_position)),
                TagType::Double => Some(self.parse_double(tag.start_position, tag.end_position)),
                TagType::BulkString => Some(self.parse_bulk_string(tag.start_position, tag.end_position)),
                TagType::BulkError => Some(self.parse_bulk_error(tag.start_position, tag.end_position)),
                TagType::VerbatimString => Some(self.parse_verbatim_string(tag.start_position, tag.end_position)),
                _ => todo!(),
            }
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }

    fn parse_boolean(&self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        if end_position - start_position != 1 {
            return Err(Error::InvalidBoolean);
        }

        match self.input.get(start_position) {
            Some(b't') => Ok(Frame::Boolean { data: true }),
            Some(b'f') => Ok(Frame::Boolean { data: false }),
            _ => Err(Error::InvalidBoolean),
        }
    }

    fn parse_simple_string(&self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::SimpleString { data }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_simple_error(&self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::SimpleError { data, }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_integer(&self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(number_str) => {
                let option = ParseIntegerOptions::new();
                let number = parse_with_options::<isize, &[u8], STANDARD>(number_str, &option)?;
                Ok(Frame::Integer { data: number })
            }
            None => Err(Error::NotComplete),
        }
    }

    fn parse_double(&self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(number_str) => {
                let option = ParseFloatOptions::new();
                let number = parse_with_options::<f64, &[u8], STANDARD>(number_str, &option)?;
                Ok(Frame::Double { data: number })
            }
            None => Err(Error::NotComplete),
        }
    }

    fn parse_bulk_string(&self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::Bulkstring { data }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_bulk_error(&self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::BulkError { data }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_verbatim_string(&self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        let encode_type = self.input.get(start_position..start_position + 3).ok_or( Error::NotComplete)?;
        let encode_type = encode_type.try_into().map_err(|_|Error::Unknown)?;
        let data = self.input.get(start_position + 3..end_position).ok_or(Error::NotComplete)?;
        Ok(Frame::VerbatimString { data: (encode_type, data) })
    }

    fn parse_array(&mut self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        let len_bytes = self.input.get(start_position..end_position).ok_or( Error::NotComplete)?;
        let options = ParseIntegerOptions::new();
        let len = parse_with_options::<usize, &[u8], STANDARD>(len_bytes, &options)?;

        let mut data = Vec::with_capacity(len);
        let mut queue = VecDeque::new();
        queue.push_back(len);

        while let Some(len) = queue.pop_front() {
            for _ in 0..len {
                match self.lexer.next() {
                    Some(Ok(tag)) => {
                        match tag.tag_type {
                            TagType::Array => {
                                let len_bytes = self.input.get(start_position..end_position).ok_or( Error::NotComplete)?;
                                let options = ParseIntegerOptions::new();
                                let len = parse_with_options::<usize, &[u8], STANDARD>(len_bytes, &options)?;
                                queue.push_back(len);
                            }
                            TagType::Double => {
                                let frame = self.parse_double(tag.start_position, tag.end_position)?;
                                data.push(frame);
                            }
                            TagType::SimpleString => {
                                let frame = self.parse_simple_string(tag.start_position, tag.end_position)?;
                                data.push(frame);
                            }
                            TagType::SimpleError => {
                                let frame = self.parse_simple_error(tag.start_position, tag.end_position)?;
                                data.push(frame);
                            }
                            TagType::BulkString => {
                                let frame = self.parse_bulk_string(tag.start_position, tag.end_position)?;
                                data.push(frame);
                            }
                            TagType::BulkError => {
                                let frame = self.parse_bulk_error(tag.start_position, tag.end_position)?;
                                data.push(frame);
                            }
                            TagType::Integer => {
                                let frame = self.parse_integer(tag.start_position, tag.end_position)?;
                                data.push(frame);
                            }
                            _=> todo!(),
                        }
                    }
                    Some(Err(e)) => {
                        return Err(e);
                    }
                    None => {
                        return Err(Error::NotComplete);
                    }
                }
            }
        }

        Ok(Frame::Array{ data })
    }
}

impl<'a> Iterator for Ast<'a> {
    type Item = Result<Frame<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_frame()
    }
}

mod test {
    use super::*;

    #[test]
    fn test_array() {
        let data = b"*3\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$3\r\nbaz\r\n";
        let mut ast = Ast::new(data);

        assert_eq!(ast.next().unwrap().unwrap(), Frame::Array{ data: vec![
            Frame::Bulkstring { data: b"foo" },
            Frame::Bulkstring{ data: b"bar" },
            Frame::Bulkstring{ data: b"baz" },
        ]});
    }
}