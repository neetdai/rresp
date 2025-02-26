use crate::common::Error;

use super::{frame::Frame, tag::Tag, Lexer};

#[derive(Debug)]
pub(crate) struct Ast<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Ast<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.lexer.remaining()
    }

    #[inline(always)]
    fn next_frame(&mut self) -> Option<Result<Frame<'a>, Error>> {
        match self.lexer.next() {
            Some(Ok(tag)) => match tag {
                Tag::BulkString(buf) => Some(Ok(Frame::BulkString(buf))),
                Tag::Null => Some(Ok(Frame::Null)),
                Tag::Integer(i) => Some(Ok(Frame::Integer(i))),
                Tag::SimpleString(buf) => Some(Ok(Frame::SimpleString(buf))),
                Tag::SimpleError(buf) => Some(Ok(Frame::SimpleError(buf))),
                Tag::Array(len) => Some(self.array_frame(len).map(|array| Frame::Array(array))),
            },
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    fn array_frame(&mut self, len: usize) -> Result<Vec<Frame<'a>>, Error> {
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            match self.next_frame() {
                Some(Ok(frame)) => {
                    list.push(frame);
                }
                Some(Err(e)) => return Err(e),
                None => return Err(Error::NotComplete),
            }
        }
        Ok(list)
    }
}

impl<'a> Iterator for Ast<'a> {
    type Item = Result<Frame<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_frame()
    }
}
