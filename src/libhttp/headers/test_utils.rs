use std::rt::io::mem::{MemReader, with_mem_writer};
use std::str;
use headers::{HeaderConvertible, HeaderValueByteIterator};

pub fn from_stream_with_str<T: HeaderConvertible>(s: &str) -> Option<T> {
    let mut bytes = s.as_bytes().into_owned();
    bytes.push_all(bytes!("\r\n/"));

    let mut reader = MemReader::new(bytes.into_owned());
    let mut iter = HeaderValueByteIterator::new(&mut reader);
    HeaderConvertible::from_stream(&mut iter)
}

pub fn to_stream_into_str<T: HeaderConvertible>(v: &T) -> ~str {
    let contents = do with_mem_writer |writer|
      { v.to_stream(writer); };
    str::from_utf8(contents)
}

// Verify that a value cannot be successfully interpreted as a header value of the specified type.
#[inline]
pub fn assert_invalid<T: HeaderConvertible>(string: &str) {
    assert_eq!(from_stream_with_str::<T>(string), None);
}

// Verify that all of the methods from the HeaderConvertible trait work correctly for the given
// valid header value and correct decoded value.
#[inline]
pub fn assert_conversion_correct<T: HeaderConvertible>(string: &'static str, value: T) {
    assert_eq!(from_stream_with_str(string), Some(value.clone()));
    let s = to_stream_into_str(&value);
    assert_eq!(s.as_slice(), string);
    let s = value.http_value();
    assert_eq!(s.as_slice(), string);
}

// Verify that from_stream interprets the given valid header value correctly.
#[inline]
pub fn assert_interpretation_correct<T: HeaderConvertible>(string: &'static str, value: T) {
    assert_eq!(from_stream_with_str(string), Some(value));
}
