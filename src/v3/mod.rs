mod ast;
mod frame;
mod lexer;
mod tag;

pub(super) use ast::Ast;
pub use frame::Frame;
pub(super) use lexer::Lexer;

use crate::{Encoder, Error, ParseIter, Parser, Remaining};

pub struct V3;

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

impl Parser for V3 {
    type Frame<'a> = (Frame<'a>, usize);

    fn parse<'a>(input: &'a [u8]) -> Result<Option<Self::Frame<'a>>, crate::Error> {
        let mut ast = Ast::new(input);
        let frame_result = ast.next().transpose();
        let remaining = ast.remaining();
        frame_result.map(|op| op.map(|frame| (frame, remaining)))
    }
}

impl ParseIter for V3 {
    type Item<'a> = Result<Frame<'a>, Error>;
    type Iter<'a> = DecodeIter<'a>;

    fn parse_iter<'a>(input: &'a [u8]) -> Self::Iter<'a> {
        DecodeIter{
            ast: Ast::new(input),
        }
    }
}
