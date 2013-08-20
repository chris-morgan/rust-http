//! The Content-Type entity header, defined in RFC 2616, Section 14.17.
use headers::serialization_utils::{push_parameters, WriterUtil};
use rfc2616::is_token;
use std::rt::io::{Reader, Writer};

#[deriving(Clone)]
pub struct MediaType {
    type_: ~str,
    subtype: ~str,
    parameters: ~[(~str, ~str)],
}

impl ToStr for MediaType {
    fn to_str(&self) -> ~str {
        // Idea:
        //let s = ~"";
        //s.push_token(self.type_);
        //s.push_char('/');
        //s.push_token(self.subtype);
        //s.push_parameters(self.parameters);
        //s
        let s = fmt!("%s/%s", self.type_, self.subtype);
        push_parameters(s, self.parameters)
    }
}

impl super::HeaderConvertible for MediaType {
    fn from_stream<T: Reader>(reader: &mut super::HeaderValueByteIterator<T>) -> Option<MediaType> {
        let type_ = reader.read_token();
        if reader.next() != '/' as u8 {
            return None;
        }
        let subtype = reader.read_token();
        match reader.read_parameters() {
            Some(parameters) if reader.is_finished() =>
                Some(MediaType {
                    type_: type_,
                    subtype: subtype,
                    parameters: parameters,
                }),
            _ => None,
        }
    }

    fn to_stream<W: Writer>(&self, writer: &mut W) {
        writer.write_token(self.type_);
        writer.write(['/' as u8]);
        writer.write_token(self.subtype);
        writer.write_parameters(self.parameters);
    }

    fn http_value(&self) -> ~str {
        self.to_str()
    }
}
