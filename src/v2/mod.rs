mod ast;
mod frame;
mod lexer;
mod tag;
mod utils;

pub(super) use ast::Ast;
pub use frame::Frame;
pub(super) use lexer::Lexer;

use crate::{
    common::{Encoder, Parser},
    EncodeWithWriter, Error, ParseIter, Remaining,
};

pub struct V2;

pub struct DecodeIter<'a> {
    ast: Ast<'a>,
}

impl<'a> Iterator for DecodeIter<'a> {
    type Item = Result<Frame<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ast.next()
    }
}

impl Remaining for DecodeIter<'_> {
    fn remaining(&self) -> usize {
        self.ast.remaining()
    }
}

impl Parser for V2 {
    type Frame<'a> = (Frame<'a>, usize);

    fn parse(input: &[u8]) -> Result<Option<Self::Frame<'_>>, crate::common::Error> {
        let buff = input.as_ref();
        let mut ast = Ast::new(buff);
        let frame_result = ast.next().transpose();
        let remainning = ast.remaining();

        frame_result.map(|op| op.map(|frame| (frame, remainning)))
    }
}

impl ParseIter for V2 {
    type Item<'a> = Result<Frame<'a>, Error>;
    type Iter<'a> = DecodeIter<'a>;

    fn parse_iter(input: &[u8]) -> Self::Iter<'_> {
        DecodeIter {
            ast: Ast::new(input),
        }
    }
}

impl Encoder for V2 {
    type Frame<'a> = Frame<'a>;
    type Item = Vec<u8>;

    fn encode(frame: Self::Frame<'_>) -> Result<Self::Item, Error> {
        Ok(frame.encode())
    }
}

impl EncodeWithWriter for V2 {
    type Frame<'a> = Frame<'a>;

    fn encode_with_writer<W>(frame: Self::Frame<'_>, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        frame.encode_with_writer(writer)
    }
}
