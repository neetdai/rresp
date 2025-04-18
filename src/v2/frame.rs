use std::{collections::VecDeque, io::{Result as IoResult, Write}};

use lexical::to_string;

use crate::{EncodeLen, common::Error};

use super::utils::CRLF;
use minivec::MiniVec;
use crate::v3::Frame as V3Frame;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub enum Frame<'a> {
    BulkString(&'a [u8]),
    Null,
    Integer(i64),
    Array(MiniVec<Frame<'a>>),
    SimpleError(&'a [u8]),
    SimpleString(&'a [u8]),
}

impl<'a> Frame<'a> {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Null => {
                let mut buf = Vec::with_capacity(5);
                buf.push(b'$');
                buf.extend_from_slice(b"-1");
                buf.extend_from_slice(&CRLF);
                buf
            }
            Self::Integer(num) => {
                let num_str = to_string(*num);
                let num_str_len = num_str.as_bytes().len();
                let mut buf = Vec::with_capacity(3 + num_str_len);
                buf.push(b':');
                buf.extend_from_slice(num_str.as_bytes());
                buf.extend_from_slice(&CRLF);
                buf
            }
            Self::SimpleError(text) => {
                let mut buf = Vec::with_capacity(3 + text.len());
                buf.push(b'-');
                buf.extend_from_slice(text);
                buf.extend_from_slice(&CRLF);
                buf
            }
            Self::SimpleString(text) => {
                let mut buf = Vec::with_capacity(3 + text.len());
                buf.push(b'+');
                buf.extend_from_slice(text);
                buf.extend_from_slice(&CRLF);
                buf
            }
            Self::BulkString(text) => {
                let text_len = text.len();
                let text_len_str = to_string(text_len);
                let mut buf = Vec::with_capacity(5 + text_len + text_len_str.len());
                buf.push(b'$');
                buf.extend_from_slice(text_len_str.as_bytes());
                buf.extend_from_slice(&CRLF);
                buf.extend_from_slice(text);
                buf.extend_from_slice(&CRLF);
                buf
            }
            Self::Array(array) => {
                let array_len = array.len();
                let array_len_str = to_string(array_len);
                let mut buf = Vec::with_capacity(3 + array_len_str.len());
                buf.push(b'*');
                buf.extend_from_slice(array_len_str.as_bytes());
                buf.extend_from_slice(&CRLF);
                for frame in array {
                    let tmp = frame.encode();
                    buf.extend(tmp.into_iter());
                }
                buf
            }
        }
    }

    pub fn encode_with_writer<W>(&self, writer: &mut W) -> IoResult<()>
    where
        W: Write,
    {
        match self {
            Self::Null => {
                writer.write_all(b"$-1\r\n")?;
            }
            Self::Integer(num) => {
                let num_str = to_string(*num);
                writer.write_all(b":")?;
                writer.write_all(num_str.as_bytes())?;
                writer.write_all(b"\r\n")?;
            }
            Self::Array(array) => {
                let array_len = array.len();
                let array_len_str = to_string(array_len);
                writer.write_all(b"*")?;
                writer.write_all(array_len_str.as_bytes())?;
                writer.write_all(b"\r\n")?;
                for frame in array {
                    frame.encode_with_writer(writer)?;
                }
            }
            Self::BulkString(text) => {
                let text_len = text.len();
                let text_len_str = to_string(text_len);
                writer.write_all(b"$")?;
                writer.write_all(text_len_str.as_bytes())?;
                writer.write_all(b"\r\n")?;
                writer.write_all(text)?;
                writer.write_all(b"\r\n")?;
            }
            Self::SimpleString(text) => {
                writer.write_all(b"+")?;
                writer.write_all(text)?;
                writer.write_all(b"\r\n")?;
            }
            Self::SimpleError(text) => {
                writer.write_all(b"-")?;
                writer.write_all(text)?;
                writer.write_all(b"\r\n")?;
            }
        }

        Ok(())
    }
}

impl<'a> EncodeLen for Frame<'a> {
    fn encode_len(&self) -> usize {
        match self {
            Self::Null => 5,
            Self::Integer(num) => {
                let num_str = to_string(*num);
                3 + num_str.len()
            }
            Self::SimpleString(text) => 3 + text.len(),
            Self::SimpleError(err) => 3 + err.len(),
            Self::BulkString(text) => 3 + text.len() + 2,
            Self::Array(array) => {
                let array_len = array.len();
                let array_len_str = to_string(array_len);
                3 + array_len_str.len() + array.iter().map(|f| f.encode_len()).sum::<usize>()
            }
        }
    }
}

impl<'a> TryFrom<V3Frame<'a>> for Frame<'a> {
    type Error = Error;

    fn try_from(value: V3Frame<'a>) -> Result<Self, Self::Error> {
        match value {
            V3Frame::Null { data: _ } => Ok(Self::Null),
            V3Frame::Integer { data, attributes } => Ok(Self::Integer(data as i64)),
            V3Frame::SimpleString { data, attributes } => Ok(Self::SimpleString(data)),
            V3Frame::SimpleError { data, attributes } => Ok(Self::SimpleError(data)),
            V3Frame::Bulkstring { data, attributes } => Ok(Self::BulkString(data)),
            V3Frame::Array { mut data, attributes } => {
                let v2_data = MiniVec::with_capacity(data.len());
                let mut stack = Vec::new();
                let queue = VecDeque::from_iter(data.drain(..));
                stack.push((v2_data, queue));

                while let Some((mut current_vec, mut current_queue)) = stack.pop() {
                    match current_queue.pop_front() {
                        Some(V3Frame::Null { data: _ }) => {
                            current_vec.push(Frame::Null);
                            stack.push((current_vec, current_queue));
                        }
                        Some(V3Frame::Integer { data, attributes }) => {
                            current_vec.push(Frame::Integer(data as i64));
                            stack.push((current_vec, current_queue));
                        }
                        Some(V3Frame::SimpleString { data, attributes }) => {
                            current_vec.push(Frame::SimpleString(data));
                            stack.push((current_vec, current_queue));
                        }
                        Some(V3Frame::SimpleError { data, attributes }) => {
                            current_vec.push(Frame::SimpleError(data));
                            stack.push((current_vec, current_queue));
                        }
                        Some(V3Frame::Array { mut data, attributes }) => {
                            let new_vec = MiniVec::with_capacity(data.len());
                            let new_queue = VecDeque::from_iter(data.drain(..));
                            stack.push((current_vec, current_queue));
                            stack.push((new_vec, new_queue));
                        }
                        Some(_) => return Err(Error::Unknown),
                        None => {
                            if stack.is_empty() {
                                let frame = Self::Array(current_vec);
                                return Ok(frame);
                            } else if let Some((parent_vec, _)) = stack.last_mut() {
                                let frame = Self::Array(current_vec);
                                parent_vec.push(frame);
                            }
                        },
                    }
                }

                Err(Error::InvalidArray)
            }
            _ => Err(Error::Unknown),
        }
    }
}

mod test {
    use super::*;
    use minivec::mini_vec;

    #[test]
    fn test_encode_null() {
        let frame = Frame::Null;
        let encoded = frame.encode();
        assert_eq!(encoded, b"$-1\r\n".to_vec());
    }

    #[test]
    fn test_encode_integer() {
        let frame = Frame::Integer(42);
        let encoded = frame.encode();
        assert_eq!(encoded, b":42\r\n".to_vec());
    }

    #[test]
    fn test_encode_simple_error() {
        let frame = Frame::SimpleError(b"ERR something went wrong");
        let encoded = frame.encode();
        assert_eq!(encoded, b"-ERR something went wrong\r\n".to_vec());
    }

    #[test]
    fn test_encode_simple_string() {
        let frame = Frame::SimpleString(b"Hello, World!");
        let encoded = frame.encode();
        assert_eq!(encoded, b"+Hello, World!\r\n".to_vec());
    }

    #[test]
    fn test_encode_blob_string() {
        let frame = Frame::BulkString(b"Hello, World!");
        let encoded = frame.encode();
        assert_eq!(encoded, b"$13\r\nHello, World!\r\n".to_vec());
    }

    #[test]
    fn test_encode_array() {
        let frame = Frame::Array(mini_vec![Frame::SimpleString(b"Hello"), Frame::Integer(42)]);
        let encoded = frame.encode();
        assert_eq!(encoded, b"*2\r\n+Hello\r\n:42\r\n".to_vec());
    }

    #[test]
    fn test_encode_with_writer() {
        let frame = Frame::Array(mini_vec![
            Frame::SimpleString(b"Hello"),
            Frame::Integer(42),
            Frame::BulkString(b"world"),
            Frame::Null,
            Frame::SimpleError(b"err"),
        ]);
        let mut buff = Vec::with_capacity(frame.encode_len());
        frame.encode_with_writer(&mut buff).unwrap();
        assert_eq!(
            buff,
            b"*5\r\n+Hello\r\n:42\r\n$5\r\nworld\r\n$-1\r\n-err\r\n".to_vec()
        );
    }
}
