mod ast;
mod frame;
mod lexer;
mod tag;

pub(super) use ast::Ast;
pub use frame::Frame;
pub(super) use lexer::Lexer;

use crate::Parser;

pub struct V3;

impl Parser for V3 {
    type Frame<'a> = Frame<'a>;

    fn parse<'a>(input: &'a [u8]) -> Result<Option<Self::Frame<'a>>, crate::Error> {
        let mut ast = Ast::new(input);
        let frame_result = ast.next().transpose();
        frame_result
    }
}