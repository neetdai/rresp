use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
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
            attributes.iter()
                .fold(0, |prev_len, (key, value)| {
                    prev_len + key.encode_len() + value.encode_len()
                })
        } else {
            0
        }
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
            Self::Null { data } => {
                3
            }
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
                text.len() + data.iter().map(|frame| frame.encode_len()).sum::<usize>() + 5 + attributes_len
            }
            Self::Map { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(data.len());
                text.len() + data.iter().map(|(key, value)| key.encode_len() + value.encode_len()).sum::<usize>() + 5 + attributes_len
            }
            Self::Set { data, attributes } => {
                let attributes_len = Self::attributes_len(attributes);
                let text = to_string(data.len());
                text.len() + data.iter().map(|frame| frame.encode_len()).sum::<usize>() + 5 + attributes_len
            }
            Self::Push { data } => {
                let text = to_string(data.len());
                text.len() + data.iter().map(|frame| frame.encode_len()).sum::<usize>() + 5
            }
            _ => 0,
        }
    }
}