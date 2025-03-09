use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use lexical::{format::STANDARD, parse_with_options, ParseFloatOptions, ParseIntegerOptions};

use crate::common::Error;

use super::{frame::Frame, lexer::Lexer, tag::TagType};

#[derive(Debug)]
pub(crate) struct Ast<'a> {
    input: &'a [u8],
    lexer: Lexer<'a>,
}

impl<'a> Ast<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            lexer: Lexer::new(input),
        }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.lexer.remaining()
    }

    fn next_frame(&mut self) -> Option<Result<Frame<'a>, Error>> {
        match self.lexer.next() {
            Some(Ok(tag)) => match tag.tag_type {
                TagType::Boolean => Some(self.parse_boolean(tag.start_position, tag.end_position)),
                TagType::SimpleString => {
                    Some(self.parse_simple_string(tag.start_position, tag.end_position))
                }
                TagType::SimpleError => {
                    Some(self.parse_simple_error(tag.start_position, tag.end_position))
                }
                TagType::Null => Some(Ok(Frame::Null { data: () })),
                TagType::Integer => Some(self.parse_integer(tag.start_position, tag.end_position)),
                TagType::Double => Some(self.parse_double(tag.start_position, tag.end_position)),
                TagType::BulkString => {
                    Some(self.parse_bulk_string(tag.start_position, tag.end_position))
                }
                TagType::BulkError => {
                    Some(self.parse_bulk_error(tag.start_position, tag.end_position))
                }
                TagType::VerbatimString => {
                    Some(self.parse_verbatim_string(tag.start_position, tag.end_position))
                }
                TagType::BigNumber => {
                    Some(self.parse_big_number(tag.start_position, tag.end_position))
                }
                TagType::Array => Some(self.parse_array(tag.start_position, tag.end_position)),
                TagType::Map => Some(self.parse_map(tag.start_position, tag.end_position)),
                TagType::Set => Some(self.parse_set(tag.start_position, tag.end_position)),
                TagType::Push => Some(self.parse_push(tag.start_position, tag.end_position)),
            },
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }

    fn parse_boolean(
        &self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        if end_position - start_position != 1 {
            return Err(Error::InvalidBoolean);
        }

        match self.input.get(start_position) {
            Some(b't') => Ok(Frame::Boolean { data: true }),
            Some(b'f') => Ok(Frame::Boolean { data: false }),
            _ => Err(Error::InvalidBoolean),
        }
    }

    fn parse_simple_string(
        &self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::SimpleString { data }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_simple_error(
        &self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::SimpleError { data }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_integer(
        &self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(number_str) => {
                let option = ParseIntegerOptions::new();
                let number = parse_with_options::<isize, &[u8], STANDARD>(number_str, &option)?;
                Ok(Frame::Integer { data: number })
            }
            None => Err(Error::NotComplete),
        }
    }

    fn parse_double(&self, start_position: usize, end_position: usize) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(number_str) => {
                let option = ParseFloatOptions::new();
                let number = parse_with_options::<f64, &[u8], STANDARD>(number_str, &option)?;
                Ok(Frame::Double { data: number })
            }
            None => Err(Error::NotComplete),
        }
    }

    fn parse_bulk_string(
        &self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::Bulkstring { data }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_bulk_error(
        &self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::BulkError { data }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_verbatim_string(
        &self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        let encode_type = self
            .input
            .get(start_position..start_position + 3)
            .ok_or(Error::NotComplete)?;
        let encode_type = encode_type.try_into().map_err(|_| Error::Unknown)?;
        let data = self
            .input
            .get(start_position + 3..end_position)
            .ok_or(Error::NotComplete)?;
        Ok(Frame::VerbatimString {
            data: (encode_type, data),
        })
    }

    fn parse_array(
        &mut self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        let len_bytes = self
            .input
            .get(start_position..end_position)
            .ok_or(Error::NotComplete)?;
        let options = ParseIntegerOptions::new();
        let len = parse_with_options::<usize, &[u8], STANDARD>(len_bytes, &options)?;

        let mut data = Vec::with_capacity(len);

        for _ in 0..len {
            match self.next_frame() {
                Some(Ok(frame)) => data.push(frame),
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            }
        }

        Ok(Frame::Array { data })
    }

    fn parse_map(
        &mut self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        let len_bytes = self
            .input
            .get(start_position..end_position)
            .ok_or(Error::NotComplete)?;
        let options = ParseIntegerOptions::new();
        let len = parse_with_options::<usize, &[u8], STANDARD>(len_bytes, &options)?;

        let mut data = HashMap::with_capacity(len);

        for _ in 0..len {
            let key = match self.next_frame() {
                Some(Ok(Frame::Map { data })) => return Err(Error::InvalidMap),
                Some(Ok(Frame::Set { data })) => return Err(Error::InvalidMap),
                Some(Ok(frame)) => frame,
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            };

            let value = match self.next_frame() {
                Some(Ok(frame)) => frame,
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            };

            data.insert(key, value);
        }

        Ok(Frame::Map { data })
    }

    fn parse_set(
        &mut self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        let len_bytes = self
            .input
            .get(start_position..end_position)
            .ok_or(Error::NotComplete)?;
        let options = ParseIntegerOptions::new();
        let len = parse_with_options::<usize, &[u8], STANDARD>(len_bytes, &options)?;

        let mut data = HashSet::with_capacity(len);

        for _ in 0..len {
            let value = match self.next_frame() {
                Some(Ok(Frame::Map { data })) => return Err(Error::InvalidSet),
                Some(Ok(Frame::Set { data })) => return Err(Error::InvalidSet),
                Some(Ok(frame)) => frame,
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            };

            data.insert(value);
        }

        Ok(Frame::Set { data })
    }

    fn parse_push(
        &mut self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        let len_bytes = self
            .input
            .get(start_position..end_position)
            .ok_or(Error::NotComplete)?;
        let options = ParseIntegerOptions::new();
        let len = parse_with_options::<usize, &[u8], STANDARD>(len_bytes, &options)?;

        let mut data = Vec::with_capacity(len);
        for _ in 0..len {
            match self.next_frame() {
                Some(Ok(frame)) => data.push(frame),
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            }
        }

        Ok(Frame::Push { data })
    }

    fn parse_big_number(
        &self,
        start_position: usize,
        end_position: usize,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::BigNumber { data }),
            None => Err(Error::NotComplete),
        }
    }
}

impl<'a> Iterator for Ast<'a> {
    type Item = Result<Frame<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_frame()
    }
}

mod test {
    use super::*;

    #[test]
    fn test_array() {
        let data = b"*3\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$3\r\nbaz\r\n";
        let mut ast = Ast::new(data);

        assert_eq!(
            ast.next().unwrap().unwrap(),
            Frame::Array {
                data: vec![
                    Frame::Bulkstring { data: b"foo" },
                    Frame::Bulkstring { data: b"bar" },
                    Frame::Bulkstring { data: b"baz" },
                ]
            }
        );

        let data = b"*1\r\n*1\r\n$3\r\nfoo\r\n";
        let mut ast = Ast::new(data);

        assert_eq!(
            ast.next().unwrap().unwrap(),
            Frame::Array {
                data: vec![Frame::Array {
                    data: vec![Frame::Bulkstring { data: b"foo" }]
                }]
            }
        )
    }

    #[test]
    fn test_map() {
        let input = b"%1\r\n$3\r\nbar\r\n$3\r\nbat\r\n";
        let mut ast = Ast::new(input);

        assert_eq!(
            ast.next().unwrap().unwrap(),
            Frame::Map {
                data: HashMap::from([(
                    Frame::Bulkstring { data: b"bar" },
                    Frame::Bulkstring { data: b"bat" }
                )]),
            }
        );

        let input = b"%1\r\n%1\r\n$3\r\nbar\r\n$3\r\nbat\r\n";
        let mut ast = Ast::new(input);

        assert_eq!(ast.next().unwrap(), Err(Error::InvalidMap));
    }
}
