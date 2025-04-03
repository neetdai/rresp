use std::fmt::Debug;

use super::Frame;
use crate::common::Error;
use lexical::{format::STANDARD, parse_with_options, ParseFloatOptions, ParseIntegerOptions};

type AstResult<'a> = Result<Frame<'a>, Error>;

pub(crate) trait ToFrame {
    fn to_frame<'a>(&self, input: &'a [u8]) -> AstResult<'a>;
}

// #[derive(Debug, PartialEq)]
// pub(crate) enum TagType {
//     BulkString,
//     SimpleString,
//     SimpleError,
//     Integer,
//     Array,
//     Null,
//     Boolean,
//     Double,
//     BigNumber,
//     BulkError,
//     VerbatimString,
//     Map,
//     Set,
//     Push,
//     Attribute,
// }

// #[derive(Debug, PartialEq)]
// pub(crate) struct Tag {
//     pub(crate) tag_type: TagType,
//     pub(crate) start_position: usize,
//     pub(crate) end_position: usize,
// }

#[derive(Debug, PartialEq)]
pub(crate) enum BasicTagType {
    BulkString,
    SimpleString,
    SimpleError,
    Integer,
    Null,
    Boolean,
    Double,
    BigNumber,
    BulkError,
    VerbatimString,
}

#[derive(Debug, PartialEq)]
pub(crate) struct BasicTag {
    pub(crate) tag_type: BasicTagType,
    pub(crate) start_position: usize,
    pub(crate) end_position: usize,
}

impl BasicTag {
    fn parse_boolean<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        if self.end_position - self.start_position != 1 {
            return Err(Error::InvalidBoolean);
        }

        match input.get(self.start_position) {
            Some(b't') => Ok(Frame::Boolean {
                data: true,
                attributes: None,
            }),
            Some(b'f') => Ok(Frame::Boolean {
                data: false,
                attributes: None,
            }),
            _ => Err(Error::InvalidBoolean),
        }
    }

    fn parse_simple_string<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        match input.get(self.start_position..self.end_position) {
            Some(s) => Ok(Frame::SimpleString {
                data: s,
                attributes: None,
            }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_simple_error<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        match input.get(self.start_position..self.end_position) {
            Some(s) => Ok(Frame::SimpleError {
                data: s,
                attributes: None,
            }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_integer<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        match input.get(self.start_position..self.end_position) {
            Some(number_str) => {
                let option = ParseIntegerOptions::new();
                let number = parse_with_options::<isize, &[u8], STANDARD>(number_str, &option)?;
                Ok(Frame::Integer {
                    data: number,
                    attributes: None,
                })
            }
            None => Err(Error::NotComplete),
        }
    }

    fn parse_big_number<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        match input.get(self.start_position..self.end_position) {
            Some(data) => Ok(Frame::BigNumber { data, attributes: None }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_double<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        match input.get(self.start_position..self.end_position) {
            Some(number_str) => {
                let option = ParseFloatOptions::new();
                let number = parse_with_options::<f64, &[u8], STANDARD>(number_str, &option)?;
                Ok(Frame::Double {
                    data: number,
                    attributes: None,
                })
            }
            None => Err(Error::NotComplete),
        }
    }

    fn parse_bulk_string<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        match input.get(self.start_position..self.end_position) {
            Some(data) => Ok(Frame::Bulkstring {
                data,
                attributes: None,
            }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_bulk_error<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        match input.get(self.start_position..self.end_position) {
            Some(data) => Ok(Frame::BulkError {
                data,
                attributes: None,
            }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_verbatim_string<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        let encode_type = input
            .get(self.start_position..self.start_position + 3)
            .ok_or(Error::NotComplete)?;
        let encode_type = encode_type.try_into().map_err(|_| Error::Unknown)?;
        let data = input
            .get(self.start_position + 3..self.end_position)
            .ok_or(Error::NotComplete)?;

        Ok(Frame::VerbatimString {
            data: (encode_type, data),
            attributes: None,
        })
    }
}

impl ToFrame for BasicTag {
    fn to_frame<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        match self.tag_type {
            BasicTagType::BulkString => self.parse_bulk_string(input),
            BasicTagType::BulkError => self.parse_bulk_error(input),
            BasicTagType::VerbatimString => self.parse_verbatim_string(input),
            BasicTagType::Boolean => self.parse_boolean(input),
            BasicTagType::Integer => self.parse_integer(input),
            BasicTagType::SimpleError => self.parse_simple_error(input),
            BasicTagType::SimpleString => self.parse_simple_string(input),
            BasicTagType::BigNumber => self.parse_big_number(input),
            BasicTagType::Double => self.parse_double(input),
            BasicTagType::Null => Ok(Frame::Null {data: ()}),
        }
    }
}

pub struct ArrayTag {
    inner: Vec<Box<dyn ToFrame>>,
}

impl ToFrame for ArrayTag {
    fn to_frame<'a>(&self, input: &'a [u8]) -> AstResult<'a> {
        let mut data = Vec::with_capacity(self.inner.len());

        for item in &self.inner {
            data.push(item.to_frame(input)?);
        }

        Ok(Frame::Array { data, attributes: None })
    }
}