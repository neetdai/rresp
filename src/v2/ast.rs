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
        let data = Vec::with_capacity(len);
        let mut stack = Vec::new();
        stack.push((data, len));

        while let Some((mut current_vec, mut current_len)) = stack.pop() {
            if current_len == 0 {
                if stack.is_empty() {
                    return Ok(current_vec);
                } else if let Some((parent_vec, parent_len)) = stack.last_mut() {
                    parent_vec.push(Frame::Array(current_vec));
                    *parent_len -= 1;
                    continue;
                }
            }

            match self.lexer.next() {
                Some(Ok(Tag::SimpleString(buf))) => {
                    current_vec.push(Frame::SimpleString(buf));
                    current_len -= 1;
                }
                Some(Ok(Tag::SimpleError(buf))) => {
                    current_vec.push(Frame::SimpleError(buf));
                    current_len -= 1;
                }
                Some(Ok(Tag::Integer(number))) => {
                    current_vec.push(Frame::Integer(number));
                    current_len -= 1;
                }
                Some(Ok(Tag::BulkString(buf))) => {
                    current_vec.push(Frame::BulkString(buf));
                    current_len -= 1;
                }
                Some(Ok(Tag::Null)) => {
                    current_vec.push(Frame::Null);
                    current_len -= 1;
                }
                Some(Ok(Tag::Array(len))) => {
                    stack.push((current_vec, current_len));
                    let new_vec = Vec::with_capacity(len);
                    stack.push((new_vec, len));
                    continue;
                }
                Some(Err(e)) => return Err(e),
                None => return Err(Error::NotComplete),
            }
            stack.push((current_vec, current_len));
        }

        Err(Error::NotComplete)
    }
}

impl<'a> Iterator for Ast<'a> {
    type Item = Result<Frame<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_frame()
    }
}
