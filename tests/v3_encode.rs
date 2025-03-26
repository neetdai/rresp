use std::collections::HashMap;

use rresp::{
    encode,
    v3::{Frame, V3}, EncodeLen,
};

#[test]
fn v3_encode() {
    let frame = Frame::SimpleString { data: b"Ok", attributes: None };
    assert_eq!(frame.encode_len(), 5);
    let encodeed = encode::<V3>(frame).unwrap();
    assert_eq!(encodeed, b"+Ok\r\n");

    let frame = Frame::SimpleString { data: b"Ok", attributes: Some(HashMap::from([(Frame::SimpleString { data: b"key", attributes: None }, Frame::SimpleString { data: b"value", attributes: None })])) };
    assert_eq!(frame.encode_len(), 23);
    let encodeed = encode::<V3>(frame).unwrap();
    assert_eq!(
        encodeed,
        b"|1\r\n+key\r\n+value\r\n+Ok\r\n"
    );

    let frame = Frame::SimpleError { data: b"err", attributes: None };
    assert_eq!(frame.encode_len(), 6);
    let encodeed = encode::<V3>(frame).unwrap();
    assert_eq!(encodeed, b"-err\r\n");

    let frame = Frame::Bulkstring { data: b"hello", attributes: None };
    assert_eq!(frame.encode_len(), 11);
    let encodeed = encode::<V3>(frame).unwrap();
    assert_eq!(encodeed, b"$5\r\nhello\r\n");

    
}