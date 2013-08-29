//! The Connection general header, defined in RFC 2616, Section 14.10.

// TODO: check if the token thing is correct or whether it's any number of tokens. Also, how and
// whether they should be interpreted (I recall its being a header name thing for legacy code,
// perhaps I should normalise header case or some such thing?)

use std::ascii::StrAsciiExt;
use std::rt::io::{Reader, Writer};

/// A value for the Connection header. Note that should it be a ``Token``, the string is in
/// normalised header case (e.g. "Keep-Alive").
#[deriving(Clone, DeepClone, Eq)]
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

impl super::CommaListHeaderConvertible for Connection;

impl super::HeaderConvertible for Connection {
    fn from_stream<T: Reader>(reader: &mut super::HeaderValueByteIterator<T>)
            -> Option<Connection> {
        let s = reader.read_token();
        let slower = normalise_header_name(s);
        if slower.as_slice() == "close" {
            Some(Close)
        } else {
            Some(Token(s))
        }
    }

    fn to_stream<T: Writer>(&self, writer: &mut T) {
        writer.write(match *self {
            Close => bytes!("close"),
            Token(ref s) => s.as_bytes(),
        });
    }

    fn http_value(&self) -> ~str {
        match *self {
            Close => ~"close",
            Token(ref s) => s.to_owned(),
        }
    }
}

#[test]
fn test_connection() {
    use headers::test_utils::*;
    assert_conversion_correct("close", ~[Close]);
    assert_conversion_correct("Foo", ~[Token(~"Foo")]);
    assert_conversion_correct("Foo, Keep-Alive", ~[Token(~"Foo"), Token(~"Keep-Alive")]);
    assert_conversion_correct("Foo, close", ~[Token(~"Foo"), Close]);
    assert_conversion_correct("close, Bar", ~[Close, Token(~"Bar")]);
    assert_interpretation_correct("close", ~[Close]);
    assert_interpretation_correct("foo", ~[Token(~"Foo")]);
    assert_interpretation_correct("foo \r\n , keep-alive", ~[Token(~"Foo"), Token(~"Keep-Alive")]);
    assert_interpretation_correct("foo,close", ~[Token(~"Foo"), Close]);
    assert_interpretation_correct("close, bar", ~[Close, Token(~"Bar")]);
    assert_invalid("foo bar");
}

#[test]
fn test_connection() {
    assert_invalid::<Connection>("foo bar");
    assert_invalid::<Connection>("foo, bar baz");
    assert_invalid::<Connection>("foo, , baz");
    assert_interpretation_correct("CLOSE", Close);
    assert_conversion_correct("close", Close);
    assert_conversion_correct("foo", Token(~"Foo"));
    assert_conversion_correct("Keep-Alive", Token(~"Keep-Alive"));
    assert_conversion_correct("close, Foo, Bar", ~[Close, Token(~"Foo"), Token("Bar")]);
    assert_interpretation_correct("foo \r\n ,BAR,close", ~[Token(~"Foo"), Token("Bar"), Close]);
}
