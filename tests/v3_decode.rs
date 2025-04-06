use minivec::mini_vec;
use std::collections::HashMap;

use rresp::{
    decode,
    v3::{Frame, V3},
};

#[test]
fn decode_v3() {
    let input = b"+OK\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::SimpleString {
                data: b"OK",
                attributes: None
            },
            5
        )
    );

    let input = b"-ERR\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::SimpleError {
                data: b"ERR",
                attributes: None
            },
            6
        )
    );

    let input = b":1\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::Integer {
                data: 1,
                attributes: None
            },
            4
        )
    );

    let input = b":-1\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::Integer {
                data: -1,
                attributes: None
            },
            5
        )
    );

    let input = b"*6\r\n:10\r\n:-1\r\n$5\r\nhello\r\n+world\r\n-err\r\n*1\r\n+ok\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::Array {
                data: mini_vec![
                    Frame::Integer {
                        data: 10,
                        attributes: None
                    },
                    Frame::Integer {
                        data: -1,
                        attributes: None
                    },
                    Frame::Bulkstring {
                        data: b"hello",
                        attributes: None
                    },
                    Frame::SimpleString {
                        data: b"world",
                        attributes: None
                    },
                    Frame::SimpleError {
                        data: b"err",
                        attributes: None
                    },
                    Frame::Array {
                        data: mini_vec![Frame::SimpleString {
                            data: b"ok",
                            attributes: None
                        }],
                        attributes: None,
                    }
                ],
                attributes: None,
            },
            48
        )
    );

    let input = b"%1\r\n$3\r\nbar\r\n*1\r\n:1\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();

    let mut data = HashMap::new();
    data.insert(
        Frame::Bulkstring {
            data: b"bar",
            attributes: None,
        },
        Frame::Array {
            data: mini_vec![Frame::Integer {
                data: 1,
                attributes: None,
            }],
            attributes: None,
        },
    );
    assert_eq!(
        (frame, remaining),
        (
            Frame::Map {
                data: data,
                attributes: None,
            },
            21
        )
    );

    let input = b"_\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::Null { data: () }, 3));

    let input = b"#t\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::Boolean {
                data: true,
                attributes: None
            },
            4
        )
    );

    let input = b"#f\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::Boolean {
                data: false,
                attributes: None
            },
            4
        )
    );

    let input = b",123.45\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::Double {
                data: 123.45,
                attributes: None
            },
            9
        )
    );

    let input = b"|1\r\n+key\r\n+value\r\n+main\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    let attributes = HashMap::from([(
        Frame::SimpleString {
            data: b"key",
            attributes: None,
        },
        Frame::SimpleString {
            data: b"value",
            attributes: None,
        },
    )]);
    assert_eq!(
        (frame, remaining),
        (
            Frame::SimpleString {
                data: b"main",
                attributes: Some(attributes)
            },
            25
        )
    );
}
