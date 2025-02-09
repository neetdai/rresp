use lexical::Error as LexicalError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("not complete")]
    NotComplete,

    #[error("unknown tag")]
    Unknown,

    #[error("invalid length")]
    SyntaxLen(#[from] LexicalError),

    #[error("invalid blob string")]
    InvalidBlobString,
}
