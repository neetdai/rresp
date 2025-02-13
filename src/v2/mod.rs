mod ast;
mod frame;
mod lexer;
mod tag;
mod utils;

use frame::Frame;
pub(super) use lexer::Lexer;
pub(super) use ast::Ast;

use crate::common::Parser;

pub struct V2 ;

impl Parser for V2 {
    type Frame<'a> = Frame<'a>;

    fn parse<T>(input: T) -> Result<Option<Self::Frame>, crate::common::Error>
        where
            T: AsRef<[u8]> {
        let buff = input.as_ref();
        let mut ast = Ast::new(buff);
        ast.next().transpose()
    }
}