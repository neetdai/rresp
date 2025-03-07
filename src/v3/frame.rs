use std::{collections::{HashMap, HashSet}, hash::Hash};

#[derive(Debug, PartialEq)]
pub enum Frame<'a> {
    SimpleString { data: &'a [u8] },
    SimpleError { data: &'a [u8] },
    Boolean { data: bool },
    Null { data: () },
    Integer { data: isize },
    Double { data: f64 },
    Bulkstring { data: &'a [u8] },
    BulkError { data: &'a [u8] },
    VerbatimString { data: ([u8; 3], &'a [u8]) },
    Array { data: Vec<Frame<'a>> },
    Map { data: HashMap<Frame<'a>, Frame<'a>> },
    Set { data: HashSet<Frame<'a>> },
}

impl<'a> Hash for Frame<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::SimpleString { data } => data.hash(state),
            Self::SimpleError { data } => data.hash(state),
            Self::Boolean { data } => data.hash(state),
            Self::Null { data } => data.hash(state),
            Self::Integer { data } => data.hash(state),
            Self::Double { data } => data.to_be_bytes().hash(state),
            Self::Bulkstring { data } => data.hash(state),
            Self::BulkError { data } => data.hash(state),
            Self::VerbatimString { data } => data.hash(state),
            Self::Array { data } => data.hash(state),
            _ => panic!("Invalid RESP3 data type to use as hash key."),
        };
    }
}

impl<'a> Eq for Frame<'a> {
}