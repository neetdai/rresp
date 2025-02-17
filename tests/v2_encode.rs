use rresp::{encode, Error, Frame, V2};

#[test]
fn test_encode_simple_string() {
    let frame = Frame::SimpleString(b"OK");
    let encoded = encode::<V2>(frame).unwrap();
    assert_eq!(encoded, b"+OK\r\n");
}

#[test]
fn test_encode_error() {
    let frame = Frame::SimpleError(b"ERR something went wrong");
    let encoded = encode::<V2>(frame).unwrap();
    assert_eq!(encoded, b"-ERR something went wrong\r\n");
}

#[test]
fn test_encode_integer() {
    let frame = Frame::Integer(123);
    let encoded = encode::<V2>(frame).unwrap();
    assert_eq!(encoded, b":123\r\n");
}

#[test]
fn test_encode_bulk_string() {
    let frame = Frame::BlobString(b"hello");
    let encoded = encode::<V2>(frame).unwrap();
    assert_eq!(encoded, b"$5\r\nhello\r\n");
}

#[test]
fn test_encode_array() {
    let frame = Frame::Array(vec![Frame::SimpleString(b"OK"), Frame::Integer(123)]);
    let encoded = encode::<V2>(frame).unwrap();
    assert_eq!(encoded, b"*2\r\n+OK\r\n:123\r\n");
}

#[test]
fn test_encode_null() {
    let frame = Frame::Null;
    let encoded = encode::<V2>(frame).unwrap();
    assert_eq!(encoded, b"$-1\r\n");
}
