//! The Connection general header, defined in RFC 2616, Section 14.10.

// TODO: check if the token thing is correct or whether it's any number of tokens. Also, how and
// whether they should be interpreted (I recall its being a header name thing for legacy code,
// perhaps I should normalise header case or some such thing?)

use std::ascii::to_ascii_lower;

pub enum Connection {
    Token(~str),
    Close,
}
impl ToStr for Connection {
    fn to_str(&self) -> ~str {
        match *self {
            Token(ref s) => s.clone(),
            Close => ~"close",
        }
    }
}

impl super::HeaderConvertible for Connection {
    fn from_stream<T: Reader>(reader: &mut HeaderValueByteIterator<T>) -> Option<Connection> {
        let s = reader.collect_to_str();
        if to_ascii_lower(s) == "close" {
            Some(Close)
        } else {
            Some(Token(s))
        }
    }

    fn to_stream<T: Writer>(&self, writer: &mut T) {
        writer.write(match self {
            Close => bytes!("close"),
            Token(ref s) => s.as_bytes(),
        });
    }

    fn http_value(&self) -> ~str {
        match self {
            Close => ~"close",
            Token(ref s) => s.to_owned(),
        }
    }
}
