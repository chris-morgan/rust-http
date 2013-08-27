use std::rt::io::mem::{MemReader, MemWriter};
use std::str;
use headers::{HeaderConvertible, HeaderValueByteIterator};

pub fn from_stream_with_str<T: HeaderConvertible>(s: &str) -> Option<T> {
    let bytes = s.as_bytes();
    let mut reader = MemReader::new(bytes.into_owned());
    reader.buf.push_all(bytes!("\r\n/"));
    let mut iter = HeaderValueByteIterator::new(&mut reader);
    HeaderConvertible::from_stream(&mut iter)
}

pub fn to_stream_into_str<T: HeaderConvertible>(v: &T) -> ~str {
    let mut writer = MemWriter::new();
    v.to_stream(&mut writer);
    str::from_bytes(writer.buf)
}
