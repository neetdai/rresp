use rresp::{decode, Error, v2::{Frame, V2}};

#[test]
fn decode_v2() {
    let input = b"+OK\r\n";
    let (frame, remaining) = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::SimpleString(b"OK"), 5));

    let input = b"-Err\r\n";
    let (frame, remaining) = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::SimpleError(b"Err"), 6));

    let input = b":1\r\n";
    let (frame, remaining) = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::Integer(1), 4));

    let input = b":-1\r\n";
    let (frame, remaining) = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::Integer(-1), 5));

    let input = b":+1\r\n";
    let (frame, remaining) = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::Integer(1), 5));

    let input = b"$5\r\nhello\r\n";
    let (frame, remaining) = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::BulkString(b"hello"), 11));

    let input = b"$-1\r\n";
    let (frame, remaining) = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::Null, 5));

    let input = b"*6\r\n:10\r\n:-1\r\n$5\r\nhello\r\n+world\r\n-err\r\n*1\r\n+ok\r\n";
    let (frame, remaining) = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!(
        (frame, remaining),
        (
            Frame::Array(vec![
                Frame::Integer(10),
                Frame::Integer(-1),
                Frame::BulkString(b"hello"),
                Frame::SimpleString(b"world"),
                Frame::SimpleError(b"err"),
                Frame::Array(vec![Frame::SimpleString(b"ok"),])
            ]),
            48
        )
    );

    let input = b":-1\r\n:1\r\n";
    let (frame, remaining) = decode::<V2>(input.as_slice()).unwrap().unwrap();
    assert_eq!((frame, remaining), (Frame::Integer(-1), 5));
}
