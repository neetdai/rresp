mod ast;
mod frame;
mod lexer;
mod tag;
mod utils;

pub(super) use ast::Ast;
pub use frame::Frame;
pub(super) use lexer::Lexer;

use crate::{common::Parser, Error, ParseIter, Remaining};

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

impl<'a> Remaining for DecodeIter<'a> {
    fn remaining(&self) -> usize {
        self.ast.remaining()
    }
}

impl Parser for V2 {
    type Frame<'a> = (Frame<'a>, usize);

    fn parse<'a>(input: &'a [u8]) -> Result<Option<Self::Frame<'a>>, crate::common::Error> {
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

    fn parse_iter<'a>(input: &'a [u8]) -> Self::Iter<'a> {
        DecodeIter {ast: Ast::new(input)}       
    }
}