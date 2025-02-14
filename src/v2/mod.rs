mod ast;
mod frame;
mod lexer;
mod tag;
mod utils;

pub(super) use ast::Ast;
pub use frame::Frame;
pub(super) use lexer::Lexer;

use crate::common::Parser;

pub struct V2;

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
