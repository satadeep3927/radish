use bytes::BytesMut;
use radish_proto::{parse, ParseError, Frame};

#[test]
fn test_parse_simple_string() {
    let mut buf = BytesMut::from("+OK\r\n");
    let (frame, consumed) = parse(&mut buf).unwrap();
    assert_eq!(frame, Frame::Simple("OK".to_string()));
    assert_eq!(consumed, 5);
}

#[test]
fn test_parse_incomplete() {
    let mut buf = BytesMut::from("+OK");
    assert_eq!(parse(&mut buf), Err(ParseError::Incomplete));
}

#[test]
fn test_parse_array_of_bulk() {
    let mut buf = BytesMut::from("*2\r\n$4\r\nECHO\r\n$11\r\nhello world\r\n");
    let (frame, consumed) = parse(&mut buf).unwrap();
    assert_eq!(
        frame,
        Frame::Array(vec![
            Frame::Bulk(bytes::Bytes::from("ECHO")),
            Frame::Bulk(bytes::Bytes::from("hello world")),
        ])
    );
    assert_eq!(consumed, 32);
}
