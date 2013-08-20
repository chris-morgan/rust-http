//! Utility functions for assisting with conversion of headers from and to the HTTP text form.

use std::vec;
use std::ascii::Ascii;
use std::rt::io::Writer;
use rfc2616::{is_token, is_token_item};

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
    for c in name.iter() {
        let c = match capitalise {
            true => c.to_ascii().to_upper(),
            false => c.to_ascii().to_lower(),
        };
        result.push(c);
        // ASCII 45 is '-': in that case, capitalise the next char
        capitalise = c.to_byte() == 45;
    }
    result.to_str_ascii()
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
    value.split_iter(',').map(|w| w.trim_left().to_owned()).collect()
}

pub fn comma_split_iter<'a>(value: &'a str)
        -> ::std::iterator::Map<'a, &'a str, &'a str, ::std::str::CharSplitIterator<'a, char>> {
    value.split_iter(',').map(|w| w.trim_left())
}

pub trait WriterUtil {
    fn write_maybe_quoted_string(&mut self, s: &str);
    fn write_quoted_string(&mut self, s: &str);
    fn write_parameter(&mut self, k: &str, v: &str);
    // TODO: &Str instead of ~str?
    fn write_parameters(&mut self, parameters: &[(~str, ~str)]);
    fn write_quality(&mut self, quality: Option<float>);
    fn write_token(&mut self, token: &str);
}

impl<W: Writer> WriterUtil for W {
    fn write_maybe_quoted_string(&mut self, s: &str) {
        if is_token(s) {
            self.write(s.as_bytes());
        } else {
            self.write_quoted_string(s);
        }
    }

    fn write_quoted_string(&mut self, s: &str) {
        self.write(['"' as u8]);
        for b in s.byte_iter() {
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

    fn write_quality(&mut self, quality: Option<float>) {
        // TODO: remove second and third decimal places if zero, and use a better quality type anyway
        match quality {
            Some(qvalue) => {
                self.write(bytes!(";q="));
                // TODO: don't use fmt! for this!
                let s = fmt!("%0.3f", qvalue);
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

pub fn parameter_split(input: &str) -> Option<~[(~str, ~str)]> {
    enum State { Start, Key, KeyEnd, Token, QuotedString, QuotedStringEscape, QuotedStringEnd }
    let mut state = Start;
    let mut output = ~[];
    let mut key = ~"";
    let mut value = ~"";
    for c in input.iter() {
        if c > '\x7f' {
            // Non-ASCII (TODO: better way to check?)
            return None;
        }
        state = match state {
            Start | Key if is_token_item(c as u8) => {
                key.push_char(c);
                Key
            },
            Key if c == '=' => KeyEnd,
            Token if is_token_item(c as u8) => {
                value.push_char(c);
                Token
            },
            KeyEnd if c == '"' => QuotedString,
            KeyEnd if is_token_item(c as u8) => {
                value.push_char(c);
                Token
            },
            QuotedString if c == '\\' => QuotedStringEscape,
            QuotedString if c == '"' => QuotedStringEnd,
            QuotedString => {
                value.push_char(c);
                QuotedStringEnd
            },
            QuotedStringEscape => {
                value.push_char(c);
                QuotedString
            },
            Token | QuotedStringEnd if c == ';' => {
                output.push((key, value));
                key = ~"";
                value = ~"";
                Start
            },
            _ => return None,
        }
    }
    match state {
        Token | QuotedStringEnd => {
            output.push((key, value));
        },
        _ => return None,
    }
    Some(output)
}

pub fn push_quality(mut s: ~str, quality: Option<float>) -> ~str {
    // TODO: remove second and third decimal places if zero, and use a better quality type anyway
    match quality {
        Some(qvalue) => {
            s.push_str(fmt!(";q=%0.3f", qvalue))
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
    for c in t.iter() {
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
    for c in s.iter() {
        state = match state {
            Start if c == '"' => Normal,
            Start => return None,
            Normal if c == '\\' => Escaping,
            Normal if c == '"' => End,
            Normal | Escaping => { output.push_char(c); Normal },
            End => return None,
        }
    }
    Some(output)
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
    use super::*;

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
    fn test_parameter_split() {
        assert_eq!(parameter_split(""), None);
        assert_eq!(parameter_split("foo=bar"), None);
        assert_eq!(parameter_split(";foo=bar"), Some(~[(~"foo", ~"bar")]));
        assert_eq!(parameter_split(";foo=not/a/token"), None);
        assert_eq!(parameter_split(";foo=bar;baz=quux"),
                   Some(~[(~"foo", ~"bar"), (~"baz", ~"quux")]));
        assert_eq!(parameter_split(";foo=bar;baz=quux;"), None);
        assert_eq!(parameter_split(";foo=bar baz;quux=zog"), None);
        assert_eq!(parameter_split(";foo=\"bar baz\";quux=zog"),
                   Some(~[(~"foo", ~"bar baz"), (~"quux", ~"zog")]));
        assert_eq!(parameter_split(";foo=\"bar baz\";quux=\"don't panic\""),
                   Some(~[(~"foo", ~"bar baz"), (~"quux", ~"don't panic")]));
    }

    #[test]
    fn test_push_quality() {
        assert_eq!(push_quality(~"foo", None), ~"foo");
        assert_eq!(push_quality(~"foo", Some(0f)), ~"foo;q=0.000");
        assert_eq!(push_quality(~"foo", Some(0.1f)), ~"foo;q=0.100");
        assert_eq!(push_quality(~"foo", Some(0.123456789f)), ~"foo;q=0.123");
        assert_eq!(push_quality(~"foo", Some(1f)), ~"foo;q=1.000");
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
                   ~"foo;bar=\"baz quux\";fuzz=zee");
    }
}
