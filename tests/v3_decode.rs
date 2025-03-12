use std::collections::HashMap;

use rresp::{
    decode,
    v3::{Frame, V3},
};

#[test]
fn decode_v3() {
    let input = b"+OK\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::SimpleString { data: b"OK" }, 5));

    let input = b"-ERR\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::SimpleError { data: b"ERR" }, 6));

    let input = b":1\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::Integer { data: 1 }, 4));

    let input = b":-1\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::Integer { data: -1 }, 5));

    let input = b"*6\r\n:10\r\n:-1\r\n$5\r\nhello\r\n+world\r\n-err\r\n*1\r\n+ok\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::Array {
                data: vec![
                    Frame::Integer { data: 10 },
                    Frame::Integer { data: -1 },
                    Frame::Bulkstring { data: b"hello" },
                    Frame::SimpleString { data: b"world" },
                    Frame::SimpleError { data: b"err" },
                    Frame::Array {
                        data: vec![Frame::SimpleString { data: b"ok" }]
                    }
                ]
            },
            48
        )
    );

    let input = b"%1\r\n$3\r\nbar\r\n*1\r\n:1\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    
    let mut data = HashMap::new();
    data.insert(Frame::Bulkstring { data: b"bar" }, Frame::Array { data: vec![Frame::Integer { data: 1 }] });
    assert_eq!(
        (frame, remaining),
        (Frame::Map {
            data: data
        }, 21)
    );

    let input = b"_\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (Frame::Null {data: ()}, 3)
    );

    let input = b"#t\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (Frame::Boolean {data: true}, 4)
    );

    let input = b"#f\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (Frame::Boolean {data: false}, 4)
    );

    let input = b",123.45\r\n";
    let (frame, remaining) = decode::<V3>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (Frame::Double {data: 123.45}, 9)
    );
}
