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
            Some(data) => Ok(Frame::SimpleString { data, }),
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
}

impl<'a> Iterator for Ast<'a> {
    type Item = Result<Frame<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_frame()
    }
}