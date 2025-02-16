use lexical::{to_string_with_options, write_integer_options::STANDARD, WriteIntegerOptions, WriteOptions};

use super::{utils::CRLF, Lexer};

#[derive(Debug, PartialEq)]
pub enum Frame<'a> {
    BlobString(&'a [u8]),
    Null,
    Integer(i64),
    Array(Vec<Frame<'a>>),
    SimpleError(&'a [u8]),
    SimpleString(&'a [u8]),
}

impl <'a> Frame<'a> {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Null => {
                let mut buf = Vec::with_capacity(5);
                buf.push(b'$');
                buf.extend_from_slice(b"-1");
                buf.extend_from_slice(&CRLF);
                buf
            },
            Self::Integer(num) => {
                let options = WriteIntegerOptions::builder().build().unwrap();
                let num_str = to_string_with_options(num, &options);
                let num_str_len = num_str.as_bytes().len();
                let mut buf = Vec::with_capacity(3 + num_str_len + 2);
                buf.push(b':');
                buf.extend_from_slice(num_str.as_bytes());
                buf.extend_from_slice(&CRLF);
                buf
            }
            _=> todo!()
        }
    }
}