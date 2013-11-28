//! Utility functions for assisting with conversion of headers from and to the HTTP text form.

use std::vec;
use std::ascii::Ascii;
use std::io::Writer;
use rfc2616::is_token;

/// Normalise an HTTP header name.
///
/// Rules:
///
/// - The first character is capitalised
/// - Any character immediately following `-` (HYPHEN-MINUS) is capitalised
/// - All other characters are made lowercase
///
/// This will fail if passed a non-ASCII name.
///
/// # Examples
///
/// ~~~ .{rust}
/// assert_eq!(normalise_header_name("foo-bar"), "Foo-Bar");
/// assert_eq!(normalise_header_name("FOO-BAR"), "Foo-Bar");
/// ~~~
pub fn normalise_header_name(name: &str) -> ~str {
    let mut result: ~[Ascii] = vec::with_capacity(name.len());
    let mut capitalise = true;
    for c in name.chars() {
        let c = match capitalise {
            true => c.to_ascii().to_upper(),
            false => c.to_ascii().to_lower(),
        };
        result.push(c);
        // ASCII 45 is '-': in that case, capitalise the next char
        capitalise = c.to_byte() == 45;
    }
    result.into_str()
}

/// Split a value on commas, as is common for HTTP headers.
///
/// This does not handle quoted-strings intelligently.
///
/// # Examples
///
/// ~~~ .{rust}
/// assert_eq!(comma_split(" en;q=0.8, en_AU, text/html"), ["en;q=0.8", "en_AU", "text/html"])
/// ~~~
pub fn comma_split(value: &str) -> ~[~str] {
    value.split(',').map(|w| w.trim_left().to_owned()).collect()
}

pub fn comma_split_iter<'a>(value: &'a str)
        -> ::std::iter::Map<'a, &'a str, &'a str, ::std::str::CharSplitIterator<'a, char>> {
    value.split(',').map(|w| w.trim_left())
}

pub trait WriterUtil: Writer {
    fn write_maybe_quoted_string(&mut self, s: &str) {
        if is_token(s) {
            self.write(s.as_bytes());
        } else {
            self.write_quoted_string(s);
        }
    }

    fn write_quoted_string(&mut self, s: &str) {
        self.write(['"' as u8]);
        for b in s.bytes() {
            if b == '\\' as u8 || b == '"' as u8 {
                self.write(['\\' as u8]);
            }
            self.write([b]);
        }
        self.write(['"' as u8]);
    }

    fn write_parameter(&mut self, k: &str, v: &str) {
        self.write(k.as_bytes());
        self.write(['=' as u8]);
        self.write_maybe_quoted_string(v);
    }

    // TODO: &Str instead of ~str?
    fn write_parameters(&mut self, parameters: &[(~str, ~str)]) {
        for &(ref k, ref v) in parameters.iter() {
            self.write([';' as u8]);
            self.write_parameter(*k, *v);
        }
    }

    fn write_quality(&mut self, quality: Option<f64>) {
        // TODO: remove second and third decimal places if zero, and use a better quality type anyway
        match quality {
            Some(qvalue) => {
                self.write(bytes!(";q="));
                // TODO: don't use format! for this!
                let s = format!("{:0.3f}", qvalue);
                self.write(s.as_bytes());
            },
            None => (),
        }
    }

    #[inline]
    fn write_token(&mut self, token: &str) {
        assert!(is_token(token));
        self.write(token.as_bytes());
    }
}

impl<W: Writer> WriterUtil for W { }

/// Join a vector of values with commas, as is common for HTTP headers.
///
/// # Examples
///
/// ~~~ .{rust}
/// assert_eq!(comma_split(["en;q=0.8", "en_AU", "text/html"]), "en;q=0.8, en_AU, text/html")
/// ~~~
#[inline]
pub fn comma_join(values: &[&str]) -> ~str {
    values.connect(", ")
}

pub fn push_quality(mut s: ~str, quality: Option<f64>) -> ~str {
    // TODO: remove second and third decimal places if zero, and use a better quality type anyway
    match quality {
        Some(qvalue) => {
            s.push_str(format!(";q={:0.3f}", qvalue))
        },
        None => (),
    }
    s
}

/// Push a ( token | quoted-string ) onto a string and return it again
pub fn push_maybe_quoted_string(mut s: ~str, t: &str) -> ~str {
    if is_token(t) {
        s.push_str(t);
        s
    } else {
        push_quoted_string(s, t)
    }
}

/// Make a string into a ( token | quoted-string ), preferring a token
pub fn maybe_quoted_string(s: ~str) -> ~str {
    if is_token(s) {
        s
    } else {
        quoted_string(s)
    }
}

/// Quote a string, to turn it into an RFC 2616 quoted-string
pub fn push_quoted_string(mut s: ~str, t: &str) -> ~str {
    let i = s.len();
    s.reserve_at_least(i + t.len() + 2);
    s.push_char('"');
    for c in t.chars() {
        if c == '\\' || c == '"' {
            s.push_char('\\');
        }
        s.push_char(c);
    }
    s.push_char('"');
    s
}

/// Quote a string, to turn it into an RFC 2616 quoted-string
pub fn quoted_string(s: &str) -> ~str {
    push_quoted_string(~"", s)
}

/// Parse a quoted-string. Returns ``None`` if the string is not a valid quoted-string.
pub fn unquote_string(s: &str) -> Option<~str> {
    enum State { Start, Normal, Escaping, End }

    let mut state = Start;
    let mut output = ~"";
    // Strings with escapes cause overallocation, but it's not worth a second pass to avoid this!
    output.reserve(s.len() - 2);
    let mut iter = s.chars();
    loop {
        state = match (state, iter.next()) {
            (Start, Some(c)) if c == '"' => Normal,
            (Start, Some(_)) => return None,
            (Normal, Some(c)) if c == '\\' => Escaping,
            (Normal, Some(c)) if c == '"' => End,
            (Normal, Some(c)) | (Escaping, Some(c)) => { output.push_char(c); Normal },
            (End, Some(_)) => return None,
            (End, None) => return Some(output),
            (_, None) => return None,
        }
    }
}

/// Parse a ( token | quoted-string ). Returns ``None`` if it is not valid.
pub fn maybe_unquote_string(s: &str) -> Option<~str> {
    if is_token(s) {
        Some(s.to_owned())
    } else {
        unquote_string(s)
    }
}

// Takes and emits the ~str instead of the &mut str for a simpler, fluid interface
pub fn push_parameter(mut s: ~str, k: &str, v: &str) -> ~str {
    s.push_str(k);
    s.push_char('=');
    push_maybe_quoted_string(s, v)
}

// TODO: &Str instead of ~str?
pub fn push_parameters(mut s: ~str, parameters: &[(~str, ~str)]) -> ~str {
    for &(ref k, ref v) in parameters.iter() {
        s.push_char(';');
        s = push_parameter(s, *k, *v);
    }
    s
}

#[cfg(test)]
mod test {
    use super::{normalise_header_name, comma_split, comma_split_iter, comma_join,
                push_quality, push_parameter, push_parameters,
                push_maybe_quoted_string, push_quoted_string, maybe_quoted_string, quoted_string,
                unquote_string, maybe_unquote_string};

    #[test]
    #[should_fail]
    fn test_normalise_header_name_fail() {
        normalise_header_name("foö-bar");
    }

    #[test]
    fn test_normalise_header_name() {
        assert_eq!(normalise_header_name("foo-bar"), ~"Foo-Bar");
        assert_eq!(normalise_header_name("FOO-BAR"), ~"Foo-Bar");
    }

    #[test]
    fn test_comma_split() {
        // Simple 0-element case
        assert_eq!(comma_split(""), ~[~""]);
        // Simple 1-element case
        assert_eq!(comma_split("foo"), ~[~"foo"]);
        // Simple 2-element case
        assert_eq!(comma_split("foo,bar"), ~[~"foo", ~"bar"]);
        // Simple >2-element case
        assert_eq!(comma_split("foo,bar,baz,quux"), ~[~"foo", ~"bar", ~"baz", ~"quux"]);
        // Doesn't handle quoted-string intelligently
        assert_eq!(comma_split("\"foo,bar\",baz"), ~[~"\"foo", ~"bar\"", ~"baz"]);
        // Doesn't do right trimming, but does left
        assert_eq!(comma_split(" foo;q=0.8 , bar/* "), ~[~"foo;q=0.8 ", ~"bar/* "]);
    }

    #[test]
    fn test_comma_split_iter() {
        // These are the same cases as in test_comma_split above.
        let s = "";
        assert_eq!(comma_split_iter(s).collect::<~[&'static str]>(), ~[""]);
        let s = "foo";
        assert_eq!(comma_split_iter(s).collect::<~[&'static str]>(), ~["foo"]);
        let s = "foo,bar";
        assert_eq!(comma_split_iter(s).collect::<~[&'static str]>(), ~["foo", "bar"]);
        let s = "foo,bar,baz,quux";
        assert_eq!(comma_split_iter(s).collect::<~[&'static str]>(), ~["foo", "bar", "baz", "quux"]);
        let s = "\"foo,bar\",baz";
        assert_eq!(comma_split_iter(s).collect::<~[&'static str]>(), ~["\"foo", "bar\"", "baz"]);
        let s = " foo;q=0.8 , bar/* ";
        assert_eq!(comma_split_iter(s).collect::<~[&'static str]>(), ~["foo;q=0.8 ", "bar/* "]);
    }

    #[test]
    fn test_comma_join() {
        assert_eq!(comma_join([""]), ~"");
        assert_eq!(comma_join(["foo"]), ~"foo");
        assert_eq!(comma_join(["foo", "bar"]), ~"foo, bar");
        assert_eq!(comma_join(["foo", "bar", "baz", "quux"]), ~"foo, bar, baz, quux");
        assert_eq!(comma_join(["\"foo,bar\"", "baz"]), ~"\"foo,bar\", baz");
        assert_eq!(comma_join([" foo;q=0.8 ", "bar/* "]), ~" foo;q=0.8 , bar/* ");
    }

    #[test]
    fn test_push_quality() {
        assert_eq!(push_quality(~"foo", None), ~"foo");
        assert_eq!(push_quality(~"foo", Some(0f64)), ~"foo;q=0.000");
        assert_eq!(push_quality(~"foo", Some(0.1f64)), ~"foo;q=0.100");
        assert_eq!(push_quality(~"foo", Some(0.123456789f64)), ~"foo;q=0.123");
        assert_eq!(push_quality(~"foo", Some(1f64)), ~"foo;q=1.000");
    }

    #[test]
    fn test_push_maybe_quoted_string() {
        assert_eq!(push_maybe_quoted_string(~"foo,", "bar"), ~"foo,bar");
        assert_eq!(push_maybe_quoted_string(~"foo,", "bar/baz"), ~"foo,\"bar/baz\"");
    }

    #[test]
    fn test_maybe_quoted_string() {
        assert_eq!(maybe_quoted_string(~"bar"), ~"bar");
        assert_eq!(maybe_quoted_string(~"bar/baz \"yay\""), ~"\"bar/baz \\\"yay\\\"\"");
    }

    #[test]
    fn test_push_quoted_string() {
        assert_eq!(push_quoted_string(~"foo,", "bar"), ~"foo,\"bar\"");
        assert_eq!(push_quoted_string(~"foo,", "bar/baz \"yay\\\""),
                   ~"foo,\"bar/baz \\\"yay\\\\\\\"\"");
    }

    #[test]
    fn test_quoted_string() {
        assert_eq!(quoted_string("bar"), ~"\"bar\"");
        assert_eq!(quoted_string("bar/baz \"yay\\\""), ~"\"bar/baz \\\"yay\\\\\\\"\"");
    }

    #[test]
    fn test_unquote_string() {
        assert_eq!(unquote_string("bar"), None);
        assert_eq!(unquote_string("\"bar\""), Some(~"bar"));
        assert_eq!(unquote_string("\"bar/baz \\\"yay\\\\\\\"\""), Some(~"bar/baz \"yay\\\""));
        assert_eq!(unquote_string("\"bar"), None);
        assert_eq!(unquote_string(" \"bar\""), None);
        assert_eq!(unquote_string("\"bar\" "), None);
        assert_eq!(unquote_string("\"bar\" \"baz\""), None);
        assert_eq!(unquote_string("\"bar/baz \\\"yay\\\\\"\""), None);
    }

    #[test]
    fn test_maybe_unquote_string() {
        assert_eq!(maybe_unquote_string("bar"), Some(~"bar"));
        assert_eq!(maybe_unquote_string("\"bar\""), Some(~"bar"));
        assert_eq!(maybe_unquote_string("\"bar/baz \\\"yay\\\\\\\"\""), Some(~"bar/baz \"yay\\\""));
        assert_eq!(maybe_unquote_string("\"bar"), None);
        assert_eq!(maybe_unquote_string(" \"bar\""), None);
        assert_eq!(maybe_unquote_string("\"bar\" "), None);
        assert_eq!(maybe_unquote_string("\"bar\" \"baz\""), None);
        assert_eq!(maybe_unquote_string("\"bar/baz \\\"yay\\\\\"\""), None);
    }

    #[test]
    fn test_push_parameter() {
        assert_eq!(push_parameter(~"foo", "bar", "baz"), ~"foobar=baz");
        assert_eq!(push_parameter(~"foo", "bar", "baz/quux"), ~"foobar=\"baz/quux\"");
    }

    #[test]
    fn test_push_parameters() {
        assert_eq!(push_parameters(~"foo", []), ~"foo");
        assert_eq!(push_parameters(~"foo", [(~"bar", ~"baz")]), ~"foo;bar=baz");
        assert_eq!(push_parameters(~"foo", [(~"bar", ~"baz/quux")]), ~"foo;bar=\"baz/quux\"");
        assert_eq!(push_parameters(~"foo", [(~"bar", ~"baz"), (~"quux", ~"fuzz")]),
                   ~"foo;bar=baz;quux=fuzz");
        assert_eq!(push_parameters(~"foo", [(~"bar", ~"baz"), (~"quux", ~"fuzz zee")]),
                   ~"foo;bar=baz;quux=\"fuzz zee\"");
        assert_eq!(push_parameters(~"foo", [(~"bar", ~"baz/quux"), (~"fuzz", ~"zee")]),
                   ~"foo;bar=\"baz/quux\";fuzz=zee");
    }
}
