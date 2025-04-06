#[derive(Debug, PartialEq)]
#[repr(u8)]
pub(crate) enum TagType {
    Attribute,
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
    AttributeBulkString,
    AttributeSimpleString,
    AttributeSimpleError,
    AttributeInteger,
    AttributeArray,
    AttributeBoolean,
    AttributeDouble,
    AttributeBigNumber,
    AttributeBulkError,
    AttributeVerbatimString,
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub(crate) struct Tag {
    pub(crate) start_position: usize,
    pub(crate) end_position: usize,
    pub(crate) tag_type: TagType,
}
