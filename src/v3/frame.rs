use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    io::{Result as IoResult, Write},
};

use lexical::to_string;

use crate::EncodeLen;

type Attributes<'a> = HashMap<Frame<'a>, Frame<'a>>;

#[derive(Debug, PartialEq)]
pub enum Frame<'a> {
    SimpleString {
        data: &'a [u8],
        attributes: Option<Attributes<'a>>,
    },
    SimpleError {
        data: &'a [u8],
        attributes: Option<Attributes<'a>>,
    },
    Boolean {
        data: bool,
        attributes: Option<Attributes<'a>>,
    },
    Null {
        data: (),
    },
    Integer {
        data: isize,
        attributes: Option<Attributes<'a>>,
    },
    Double {
        data: f64,
        attributes: Option<Attributes<'a>>,
    },
    Bulkstring {
        data: &'a [u8],
        attributes: Option<Attributes<'a>>,
    },
    BulkError {
        data: &'a [u8],
        attributes: Option<Attributes<'a>>,
    },
    VerbatimString {
        data: ([u8; 3], &'a [u8]),
        attributes: Option<Attributes<'a>>,
    },
    Array {
        data: Vec<Frame<'a>>,
        attributes: Option<Attributes<'a>>,
    },
    Map {
        data: HashMap<Frame<'a>, Frame<'a>>,
        attributes: Option<Attributes<'a>>,
    },
    Set {
        data: HashSet<Frame<'a>>,
        attributes: Option<Attributes<'a>>,
    },
    Push {
        data: Vec<Frame<'a>>,
    },
    BigNumber {
        data: &'a [u8],
        attributes: Option<Attributes<'a>>,
    },
}

impl<'a> Hash for Frame<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::SimpleString { data, attributes } => data.hash(state),
            Self::SimpleError { data, attributes } => data.hash(state),
            Self::Boolean { data, attributes } => data.hash(state),
            Self::Null { data } => data.hash(state),
            Self::Integer { data, attributes } => data.hash(state),
            Self::Double { data, attributes } => data.to_be_bytes().hash(state),
            Self::Bulkstring { data, attributes } => data.hash(state),
            Self::BulkError { data, attributes } => data.hash(state),
            Self::VerbatimString { data, attributes } => data.hash(state),
            Self::BigNumber { data, attributes } => data.hash(state),
            Self::Array { data, attributes } => data.hash(state),
            _ => panic!("Invalid RESP3 data type to use as hash key."),
        };
    }
}

impl<'a> Eq for Frame<'a> {}

impl<'a> Frame<'a> {
    fn attributes_len(attributes: &Option<Attributes<'a>>) -> usize {
        if let Some(attributes) = attributes {
            let attributes_len = attributes.len();
            let attributes_len_text = to_string(attributes_len);
            attributes
                .iter()
                .fold(3 + attributes_len_text.len(), |prev_len, (key, value)| {
                    prev_len + key.encode_len() + value.encode_len()
                })
        } else {
            0
        }
    }

    fn attibutes_encode<W>(attributes: &Option<Attributes<'a>>, writer: &mut W) -> IoResult<()>
    where
        W: Write,
    {
        if let Some(attributes) = attributes {
            let attributes_len = attributes.len();
            let attributes_len_text = to_string(attributes_len);
            writer.write(b"|")?;
            writer.write(attributes_len_text.as_bytes())?;
            writer.write(b"\r\n")?;
            attributes.iter().try_for_each(|(key, value)| {
                key.encode_with_writer(writer)?;
                value.encode_with_writer(writer)?;
                Ok(())
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> Frame<'a> {
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(self.encode_len());
        self.encode_with_writer(&mut buffer);
        buffer
    }

    pub fn encode_with_writer<W>(&self, writer: &mut W) -> IoResult<()>
    where
        W: Write,
    {
        match self {
            Self::SimpleString { data, attributes } => {
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"+")?;
                writer.write(&data)?;
                writer.write(b"\r\n")?;
            }
            Self::SimpleError { data, attributes } => {
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"-")?;
                writer.write(&data)?;
                writer.write(b"\r\n")?;
            }
            Self::Boolean { data, attributes } => {
                let bool_text = if *data { b"t" } else { b"f" };
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"#")?;
                writer.write(bool_text)?;
                writer.write(b"\r\n")?;
            }
            Self::Null { data: _ } => {
                writer.write(b"_\r\n")?;
            }
            Self::Integer { data, attributes } => {
                let text = to_string(*data);
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b":")?;
                writer.write(text.as_bytes())?;
                writer.write(b"\r\n")?;
            }
            Self::Double { data, attributes } => {
                let text = to_string(*data);
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b",")?;
                writer.write(text.as_bytes())?;
                writer.write(b"\r\n")?;
            }
            Self::Bulkstring { data, attributes } => {
                let data_len = data.len();
                let data_len_text = to_string(data_len);
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"$")?;
                writer.write(data_len_text.as_bytes())?;
                writer.write(b"\r\n")?;
                writer.write(data)?;
                writer.write(b"\r\n")?;
            }
            Self::BulkError { data, attributes } => {
                let data_len = data.len();
                let data_len_text = to_string(data_len);
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"!")?;
                writer.write(data_len_text.as_bytes())?;
                writer.write(b"\r\n")?;
                writer.write(data)?;
                writer.write(b"\r\n")?;
            }
            Self::BigNumber { data, attributes } => {
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"(")?;
                writer.write(data)?;
                writer.write(b"\r\n")?;
            }
            Self::VerbatimString { data, attributes } => {
                let data_len = data.1.len();
                let data_len_text = to_string(data_len + 4);
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"=")?;
                writer.write(data_len_text.as_bytes())?;
                writer.write(data.0.as_slice())?;
                writer.write(b":")?;
                writer.write(data.1)?;
                writer.write(b"\r\n")?;
            }
            Self::Array { data, attributes } => {
                let data_len = data.len();
                let data_len_text = to_string(data_len);
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"*")?;
                writer.write(data_len_text.as_bytes())?;
                writer.write(b"\r\n")?;
                for frame in data {
                    frame.encode_with_writer(writer)?;
                }
            }
            Self::Map { data, attributes } => {
                let data_len = data.len();
                let data_len_text = to_string(data_len);
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"%")?;
                writer.write(data_len_text.as_bytes())?;
                writer.write(b"\r\n")?;
                for (key, value) in data {
                    key.encode_with_writer(writer)?;
                    value.encode_with_writer(writer)?;
                }
            }
            Self::Set { data, attributes } => {
                let data_len = data.len();
                let data_len_text = to_string(data_len);
                Self::attibutes_encode(attributes, writer)?;
                writer.write(b"~")?;
                writer.write(data_len_text.as_bytes())?;
                writer.write(b"\r\n")?;
                for frame in data {
                    frame.encode_with_writer(writer)?;
                }
            }
            Self::Push { data } => {
                let data_len = data.len();
                let data_len_text = to_string(data_len);
                writer.write(b">")?;
                writer.write(data_len_text.as_bytes())?;
                writer.write(b"\r\n")?;
                for frame in data {
                    frame.encode_with_writer(writer)?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> EncodeLen for Frame<'a> {
    fn encode_len(&self) -> usize {
        match self {
            Self::SimpleString { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                data.len() + 3 + attributes_len
            }
            Self::SimpleError { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                data.len() + 3 + attributes_len
            }
            Self::Boolean { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                4 + attributes_len
            }
            Self::Null { data } => 3,
            Self::Integer { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(*data);
                text.len() + 3 + attributes_len
            }
            Self::Double { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(*data);
                text.len() + 3 + attributes_len
            }
            Self::Bulkstring { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(data.len());
                text.len() + data.len() + 5 + attributes_len
            }
            Self::BulkError { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(data.len());
                text.len() + data.len() + 5 + attributes_len
            }
            Self::VerbatimString { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(data.1.len());
                text.len() + data.1.len() + 8 + attributes_len
            }
            Self::Array { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(data.len());
                text.len()
                    + data.iter().map(|frame| frame.encode_len()).sum::<usize>()
                    + 5
                    + attributes_len
            }
            Self::Map { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(data.len());
                text.len()
                    + data
                        .iter()
                        .map(|(key, value)| key.encode_len() + value.encode_len())
                        .sum::<usize>()
                    + 5
                    + attributes_len
            }
            Self::Set { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(data.len());
                text.len()
                    + data.iter().map(|frame| frame.encode_len()).sum::<usize>()
                    + 5
                    + attributes_len
            }
            Self::Push { data } => {
                let text = to_string(data.len());
                text.len() + data.iter().map(|frame| frame.encode_len()).sum::<usize>() + 5
            }
            _ => 0,
        }
    }
}
