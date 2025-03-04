#[derive(Debug, PartialEq)]
pub(crate) enum TagType {
    BulkString,
    SimpleString,
    SimpleError,
    Integer,
    Array,
    Null,
    Boolean,
    Double,
    BigNumber,
    BulkError,
    VerbatimString,
    Map,
    Set,
    Push,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Tag {
    pub(crate) tag_type: TagType,
    pub(crate) start_position: usize,
    pub(crate) end_position: usize,
}
