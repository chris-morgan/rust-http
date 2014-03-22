//! The Connection general header, defined in RFC 2616, Section 14.10.

// TODO: check if the token thing is correct or whether it's any number of tokens. Also, how and
// whether they should be interpreted (I recall its being a header name thing for legacy code,
// perhaps I should normalise header case or some such thing?)

use std::fmt;
use std::io::IoResult;
use headers::serialization_utils::normalise_header_name;

/// A value for the Connection header. Note that should it be a ``Token``, the string is in
/// normalised header case (e.g. "Keep-Alive").
#[deriving(Clone, Eq)]
pub enum Connection {
    Token(~str),
    Close,
}

impl fmt::Show for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.buf.write(match *self {
            Token(ref s) => s.as_bytes(),
            Close => "close".as_bytes(),
        })
    }
}

impl super::CommaListHeaderConvertible for Connection {}

impl super::HeaderConvertible for Connection {
    fn from_stream<R: Reader>(reader: &mut super::HeaderValueByteIterator<R>)
            -> Option<Connection> {
        let s = match reader.read_token() {
            Some(s) => normalise_header_name(s),
            None => return None,
        };
        if s.as_slice() == "Close" {
            Some(Close)
        } else {
            Some(Token(s))
        }
    }

    fn to_stream<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
        writer.write(match *self {
            Close => "close".as_bytes(),
            Token(ref s) => s.as_bytes(),
        })
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
    use headers::test_utils::{assert_conversion_correct,
                              assert_interpretation_correct,
                              assert_invalid};
    assert_conversion_correct("close", vec!(Close));
    assert_conversion_correct("Foo", vec!(Token(~"Foo")));
    assert_conversion_correct("Foo, Keep-Alive", vec!(Token(~"Foo"), Token(~"Keep-Alive")));
    assert_conversion_correct("Foo, close", vec!(Token(~"Foo"), Close));
    assert_conversion_correct("close, Bar", vec!(Close, Token(~"Bar")));

    assert_interpretation_correct("close", vec!(Close));
    assert_interpretation_correct("foo", vec!(Token(~"Foo")));
    assert_interpretation_correct("close \r\n , keep-ALIVE", vec!(Close, Token(~"Keep-Alive")));
    assert_interpretation_correct("foo,close", vec!(Token(~"Foo"), Close));
    assert_interpretation_correct("close, bar", vec!(Close, Token(~"Bar")));
    assert_interpretation_correct("CLOSE", Close);

    assert_invalid::<Vec<Connection>>("foo bar");
    assert_invalid::<Vec<Connection>>("foo bar");
    assert_invalid::<Vec<Connection>>("foo, bar baz");
    assert_invalid::<Vec<Connection>>("foo, , baz");
}
