//! The Connection general header, defined in RFC 2616, Section 14.10.

// TODO: check if the token thing is correct or whether it's any number of tokens. Also, how and
// whether they should be interpreted (I recall its being a header name thing for legacy code,
// perhaps I should normalise header case or some such thing?)

use std::rt::io::{Reader, Writer};
use headers::serialization_utils::normalise_header_name;

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

impl super::CommaListHeaderConvertible for Connection {}

impl super::HeaderConvertible for Connection {
    fn from_stream<T: Reader>(reader: &mut super::HeaderValueByteIterator<T>)
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
}
#[test]
fn test_connection_2() {
    use headers::test_utils::*;
    assert_conversion_correct("Foo", ~[Token(~"Foo")]);
}
#[test]
fn test_connection_3() {
    use headers::test_utils::*;
    assert_conversion_correct("Foo, Keep-Alive", ~[Token(~"Foo"), Token(~"Keep-Alive")]);
}
#[test]
fn test_connection_4() {
    use headers::test_utils::*;
    assert_conversion_correct("Foo, close", ~[Token(~"Foo"), Close]);
}
#[test]
fn test_connection_5() {
    use headers::test_utils::*;
    assert_conversion_correct("close, Bar", ~[Close, Token(~"Bar")]);
}
#[test]
fn test_connection_6() {
    use headers::test_utils::*;
    assert_interpretation_correct("close", ~[Close]);
}
#[test]
fn test_connection_7() {
    use headers::test_utils::*;
    assert_interpretation_correct("foo", ~[Token(~"Foo")]);
}
#[test]
#[ignore(reason="lws collapse bug")]
fn test_connection_8() {
    use headers::test_utils::*;
    assert_interpretation_correct("close \r\n , keep-ALIVE", ~[Close, Token(~"Keep-Alive")]);
}
#[test]
fn test_connection_9() {
    use headers::test_utils::*;
    assert_interpretation_correct("foo,close", ~[Token(~"Foo"), Close]);
}
#[test]
fn test_connection_10() {
    use headers::test_utils::*;
    assert_interpretation_correct("close, bar", ~[Close, Token(~"Bar")]);
}
#[test]
fn test_connection_11() {
    use headers::test_utils::*;
    assert_interpretation_correct("CLOSE", Close);
}
#[test]
#[ignore(reason="lws collapse bug")]
fn test_connection_12() {
    use headers::test_utils::*;
    assert_invalid::<~[Connection]>("foo bar");
}
#[test]
#[ignore(reason="lws collapse bug")]
fn test_connection_13() {
    use headers::test_utils::*;
    assert_invalid::<~[Connection]>("foo bar");
}
#[test]
#[ignore(reason="lws collapse bug")]
fn test_connection_14() {
    use headers::test_utils::*;
    assert_invalid::<~[Connection]>("foo, bar baz");
}
#[test]
fn test_connection_15() {
    use headers::test_utils::*;
    assert_invalid::<~[Connection]>("foo, , baz");
}
