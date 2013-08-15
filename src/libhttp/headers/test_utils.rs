use std::rt::io::{Reader, Writer, Stream};
use std::rt::io::mem::{MemReader, MemWriter};
use std::str;

pub fn from_stream_with_str<T: HeaderConvertible<MemReader>>(s: &str) -> Option<T> {
    let bytes = s.as_bytes();
    let mut reader = MemReader::new(bytes.into_owned());
    reader.buf.push_all(s.as_bytes());
    HeaderConvertible::from_stream(reader)
}

pub fn to_stream_into_str<T: HeaderConvertible<MemWriter>>(v: &T)) -> ~str {
    let mut writer = MemWriter::new();
    v.to_stream(writer);
    str::from_bytes(writer.buf)
}
