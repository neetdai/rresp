use rresp::{decode, Frame, V2, Error};

#[test]
fn decode_v2() {
    let input = b"+OK\r\n";
    let frame = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!(frame, Frame::SimpleString(b"OK"));

    let input = b"-Err\r\n";
    let frame = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!(frame, Frame::SimpleError(b"Err"));

    let input = b":1\r\n";
    let frame = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!(frame, Frame::Integer(1));

    let input = b":-1\r\n";
    let frame = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!(frame, Frame::Integer(-1));

    let input = b":+1\r\n";
    let frame = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!(frame, Frame::Integer(1));

    let input = b"$5\r\nhello\r\n";
    let frame = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!(frame, Frame::BlobString(b"hello"));

    let input = b"$-1\r\n";
    let frame = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!(frame, Frame::Null);

    let input = b"*6\r\n:10\r\n:-1\r\n$5\r\nhello\r\n+world\r\n-err\r\n*1\r\n+ok\r\n";
    let frame = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!(frame, Frame::Array(vec![
        Frame::Integer(10),
        Frame::Integer(-1),
        Frame::BlobString(b"hello"),
        Frame::SimpleString(b"world"),
        Frame::SimpleError(b"err"),
        Frame::Array(vec![
            Frame::SimpleString(b"ok"),
        ])
    ]));
}