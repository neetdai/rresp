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
            Some(data) => Ok(Frame::SimpleString { data: data }),
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