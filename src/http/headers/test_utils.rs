use std::rt::io::Decorator;
use std::rt::io::mem::{MemReader, MemWriter};
use std::str;
use headers::{HeaderConvertible, HeaderValueByteIterator};

pub fn from_stream_with_str<T: HeaderConvertible>(s: &str) -> Option<T> {
    let bytes = s.as_bytes();
    let mut reader = MemReader::new(bytes.into_owned());
    reader.inner_mut_ref().push_all(bytes!("\r\n/"));
    let mut iter = HeaderValueByteIterator::new(&mut reader);
    HeaderConvertible::from_stream(&mut iter)
}

pub fn to_stream_into_str<T: HeaderConvertible>(v: &T) -> ~str {
    let mut writer = MemWriter::new();
    v.to_stream(&mut writer);
    str::from_utf8(writer.inner_ref().as_slice())
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
