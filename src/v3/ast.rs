use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    iter::Peekable,
};

use crate::common::Error;
use lexical::{format::STANDARD, parse_with_options, ParseFloatOptions, ParseIntegerOptions};
use minivec::MiniVec;

use super::{frame::Frame, lexer::Lexer, tag::TagType};

#[derive(Debug)]
pub(crate) struct Ast<'a> {
    input: &'a [u8],
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Ast<'a> {
    pub(crate) fn new(input: &'a [u8]) -> Self {
        let lexer = Lexer::new(input);
        let peek = lexer.peekable();
        Self { input, lexer: peek }
    }

    fn next_frame(&mut self) -> Option<Result<Frame<'a>, Error>> {
        match self.lexer.next() {
            Some(Ok(tag)) => match tag.tag_type {
                TagType::Boolean => {
                    Some(self.parse_boolean(tag.start_position, tag.end_position, None))
                }
                TagType::SimpleString => {
                    Some(self.parse_simple_string(tag.start_position, tag.end_position, None))
                }
                TagType::SimpleError => {
                    Some(self.parse_simple_error(tag.start_position, tag.end_position, None))
                }
                TagType::Null => Some(Ok(Frame::Null { data: () })),
                TagType::Integer => {
                    Some(self.parse_integer(tag.start_position, tag.end_position, None))
                }
                TagType::Double => {
                    Some(self.parse_double(tag.start_position, tag.end_position, None))
                }
                TagType::BulkString => {
                    Some(self.parse_bulk_string(tag.start_position, tag.end_position, None))
                }
                TagType::BulkError => {
                    Some(self.parse_bulk_error(tag.start_position, tag.end_position, None))
                }
                TagType::VerbatimString => {
                    Some(self.parse_verbatim_string(tag.start_position, tag.end_position, None))
                }
                TagType::BigNumber => {
                    Some(self.parse_big_number(tag.start_position, tag.end_position, None))
                }
                TagType::Array => {
                    Some(self.parse_array(tag.start_position, tag.end_position, None))
                }
                TagType::Map => Some(self.parse_map(tag.start_position, tag.end_position, None)),
                TagType::Set => Some(self.parse_set(tag.start_position, tag.end_position, None)),
                TagType::Push => Some(self.parse_push(tag.start_position, tag.end_position)),
                TagType::Attribute => {
                    Some(self.parse_attribute(tag.start_position, tag.end_position))
                }
                _ => Some(Err(Error::Unknown)),
            },
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }

    #[inline(always)]
    fn parse_boolean(
        &self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
    ) -> Result<Frame<'a>, Error> {
        if end_position - start_position != 1 {
            return Err(Error::InvalidBoolean);
        }

        match self.input.get(start_position) {
            Some(b't') => Ok(Frame::Boolean {
                data: true,
                attributes,
            }),
            Some(b'f') => Ok(Frame::Boolean {
                data: false,
                attributes,
            }),
            _ => Err(Error::InvalidBoolean),
        }
    }

    #[inline(always)]
    fn parse_simple_string(
        &self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::SimpleString { data, attributes }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_simple_error(
        &self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::SimpleError { data, attributes }),
            None => Err(Error::NotComplete),
        }
    }

    #[inline(always)]
    fn parse_integer(
        &self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(number_str) => {
                let option = ParseIntegerOptions::new();
                let number = parse_with_options::<isize, &[u8], STANDARD>(number_str, &option)?;
                Ok(Frame::Integer {
                    data: number,
                    attributes,
                })
            }
            None => Err(Error::NotComplete),
        }
    }

    #[inline(always)]
    fn parse_double(
        &self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(number_str) => {
                let option = ParseFloatOptions::new();
                let number = parse_with_options::<f64, &[u8], STANDARD>(number_str, &option)?;
                Ok(Frame::Double {
                    data: number,
                    attributes,
                })
            }
            None => Err(Error::NotComplete),
        }
    }

    #[inline(always)]
    fn parse_bulk_string(
        &self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::BulkString { data, attributes }),
            None => Err(Error::NotComplete),
        }
    }

    #[inline(always)]
    fn parse_bulk_error(
        &self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::BulkError { data, attributes }),
            None => Err(Error::NotComplete),
        }
    }

    #[inline(always)]
    fn parse_verbatim_string(
        &self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
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
            attributes,
        })
    }

    fn parse_array(
        &mut self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
    ) -> Result<Frame<'a>, Error> {
        let len_bytes = self
            .input
            .get(start_position..end_position)
            .ok_or(Error::NotComplete)?;
        let options = ParseIntegerOptions::new();
        let len = parse_with_options::<usize, &[u8], STANDARD>(len_bytes, &options)?;

        let data = MiniVec::with_capacity(len);

        let mut stack = Vec::new();
        stack.push((data, len));

        while let Some((mut current_vec, current_len)) = stack.pop() {
            if current_len == 0 {
                if stack.is_empty() {
                    return Ok(Frame::Array {
                        data: current_vec,
                        attributes,
                    });
                } else if let Some((parent_vec, parent_len)) = stack.last_mut() {
                    parent_vec.push(Frame::Array {
                        data: current_vec,
                        attributes: None,
                    });
                    *parent_len -= 1;
                    continue;
                }
            }

            match self.lexer.next() {
                Some(Ok(tag)) => match tag.tag_type {
                    TagType::Boolean => {
                        let frame =
                            self.parse_boolean(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::SimpleString => {
                        let frame =
                            self.parse_simple_string(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::SimpleError => {
                        let frame =
                            self.parse_simple_error(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::Null => {
                        let frame = Frame::Null { data: () };
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::Integer => {
                        let frame =
                            self.parse_integer(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::Double => {
                        let frame =
                            self.parse_double(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::BulkString => {
                        let frame =
                            self.parse_bulk_string(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::BulkError => {
                        let frame =
                            self.parse_bulk_error(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::VerbatimString => {
                        let frame =
                            self.parse_verbatim_string(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::BigNumber => {
                        let frame =
                            self.parse_big_number(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::Array => {
                        let new_len_bytes = self
                            .input
                            .get(tag.start_position..tag.end_position)
                            .ok_or(Error::NotComplete)?;
                        let new_len =
                            parse_with_options::<usize, &[u8], STANDARD>(new_len_bytes, &options)?;

                        let new_array = MiniVec::with_capacity(new_len);
                        stack.push((current_vec, current_len));
                        stack.push((new_array, new_len));
                    }
                    TagType::Map => {
                        let frame = self.parse_map(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::Set => {
                        let frame = self.parse_set(tag.start_position, tag.end_position, None)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::Push => {
                        let frame = self.parse_push(tag.start_position, tag.end_position)?;
                        current_vec.push(frame);
                        stack.push((current_vec, current_len - 1));
                    }
                    TagType::Attribute => return Err(Error::InvalidBulkString),
                    _ => return Err(Error::Unknown),
                },
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            }
        }

        Err(Error::NotComplete)
    }

    fn parse_map(
        &mut self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
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
                Some(Ok(Frame::Map { data, attributes })) => return Err(Error::InvalidMap),
                Some(Ok(Frame::Set { data, attributes })) => return Err(Error::InvalidMap),
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

        Ok(Frame::Map { data, attributes })
    }

    fn parse_set(
        &mut self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
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
                Some(Ok(Frame::Map { data, attributes })) => return Err(Error::InvalidSet),
                Some(Ok(Frame::Set { data, attributes })) => return Err(Error::InvalidSet),
                Some(Ok(frame)) => frame,
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            };

            data.insert(value);
        }

        Ok(Frame::Set { data, attributes })
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

        let mut data = MiniVec::with_capacity(len);
        for _ in 0..len {
            match self.next_frame() {
                Some(Ok(frame)) => {
                    data.push(frame);
                }
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            }
        }

        Ok(Frame::Push { data })
    }

    #[inline(always)]
    fn parse_big_number(
        &self,
        start_position: usize,
        end_position: usize,
        attributes: Option<HashMap<Frame<'a>, Frame<'a>>>,
    ) -> Result<Frame<'a>, Error> {
        match self.input.get(start_position..end_position) {
            Some(data) => Ok(Frame::BigNumber { data, attributes }),
            None => Err(Error::NotComplete),
        }
    }

    fn parse_attribute(
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

        let mut attributes = HashMap::with_capacity(len);
        let mut func = || -> Option<Result<Frame<'a>, Error>> {
            match self.lexer.next() {
                Some(Ok(tag)) => match tag.tag_type {
                    TagType::Boolean => {
                        Some(self.parse_boolean(tag.start_position, tag.end_position, None))
                    }
                    TagType::SimpleString => {
                        Some(self.parse_simple_string(tag.start_position, tag.end_position, None))
                    }
                    TagType::SimpleError => {
                        Some(self.parse_simple_error(tag.start_position, tag.end_position, None))
                    }
                    TagType::Null => Some(Ok(Frame::Null { data: () })),
                    TagType::Integer => {
                        Some(self.parse_integer(tag.start_position, tag.end_position, None))
                    }
                    TagType::Double => {
                        Some(self.parse_double(tag.start_position, tag.end_position, None))
                    }
                    TagType::BulkString => {
                        Some(self.parse_bulk_string(tag.start_position, tag.end_position, None))
                    }
                    TagType::BulkError => {
                        Some(self.parse_bulk_error(tag.start_position, tag.end_position, None))
                    }
                    TagType::VerbatimString => {
                        Some(self.parse_verbatim_string(tag.start_position, tag.end_position, None))
                    }
                    TagType::BigNumber => {
                        Some(self.parse_big_number(tag.start_position, tag.end_position, None))
                    }
                    TagType::Array => {
                        Some(self.parse_array(tag.start_position, tag.end_position, None))
                    }
                    TagType::Map => Some(Err(Error::InvalidMap)),
                    TagType::Set => Some(Err(Error::InvalidSet)),
                    _ => Some(Err(Error::Unknown)),
                },
                Some(Err(err)) => Some(Err(err)),
                None => None,
            }
        };
        for _ in 0..len {
            let key = match func() {
                Some(Ok(frame)) => frame,
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            };

            let value = match func() {
                Some(Ok(frame)) => frame,
                Some(Err(err)) => return Err(err),
                None => return Err(Error::NotComplete),
            };
            attributes.insert(key, value);
        }
        let attributes = Some(attributes);
        match self.lexer.next() {
            Some(Ok(tag)) => match tag.tag_type {
                TagType::SimpleString => {
                    self.parse_simple_string(tag.start_position, tag.end_position, attributes)
                }
                TagType::SimpleError => {
                    self.parse_simple_error(tag.start_position, tag.end_position, attributes)
                }
                TagType::Integer => {
                    self.parse_integer(tag.start_position, tag.end_position, attributes)
                }
                TagType::BigNumber => {
                    self.parse_big_number(tag.start_position, tag.end_position, attributes)
                }
                TagType::Double => {
                    self.parse_double(tag.start_position, tag.end_position, attributes)
                }
                TagType::Boolean => {
                    self.parse_boolean(tag.start_position, tag.end_position, attributes)
                }
                TagType::BulkString => {
                    self.parse_bulk_string(tag.start_position, tag.end_position, attributes)
                }
                TagType::BulkError => {
                    self.parse_bulk_error(tag.start_position, tag.end_position, attributes)
                }
                TagType::VerbatimString => {
                    self.parse_verbatim_string(tag.start_position, tag.end_position, attributes)
                }
                _ => return Err(Error::Unknown),
            },
            Some(Err(e)) => return Err(e),
            None => return Err(Error::NotComplete),
        }
    }
}

impl<'a> Iterator for Ast<'a> {
    type Item = Result<Frame<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_frame()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lexer.size_hint()
    }
}

mod test {
    use super::*;
    use minivec::mini_vec;

    #[test]
    fn test_array() {
        let data = b"*3\r\n$3\r\nfoo\r\n$3\r\nbar\r\n$3\r\nbaz\r\n";
        let mut ast = Ast::new(data);

        assert_eq!(
            ast.next().unwrap().unwrap(),
            Frame::Array {
                data: mini_vec![
                    Frame::BulkString {
                        data: b"foo",
                        attributes: None
                    },
                    Frame::BulkString {
                        data: b"bar",
                        attributes: None
                    },
                    Frame::BulkString {
                        data: b"baz",
                        attributes: None
                    },
                ],
                attributes: None,
            }
        );

        let data = b"*1\r\n*1\r\n$3\r\nfoo\r\n";
        let mut ast = Ast::new(data);

        assert_eq!(
            ast.next().unwrap().unwrap(),
            Frame::Array {
                data: mini_vec![Frame::Array {
                    data: mini_vec![Frame::BulkString {
                        data: b"foo",
                        attributes: None
                    }],
                    attributes: None,
                }],
                attributes: None,
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
                    Frame::BulkString {
                        data: b"bar",
                        attributes: None
                    },
                    Frame::BulkString {
                        data: b"bat",
                        attributes: None
                    }
                )]),
                attributes: None,
            }
        );

        let input = b"%1\r\n%1\r\n$3\r\nbar\r\n$3\r\nbat\r\n";
        let mut ast = Ast::new(input);

        assert_eq!(ast.next().unwrap(), Err(Error::InvalidMap));
    }

    #[test]
    fn test_push() {
        let input = b">1\r\n$3\r\nbar\r\n";
        let mut ast = Ast::new(input);

        assert_eq!(
            ast.next().unwrap(),
            Ok(Frame::Push {
                data: mini_vec![Frame::BulkString {
                    data: b"bar",
                    attributes: None
                }]
            })
        )
    }
}
