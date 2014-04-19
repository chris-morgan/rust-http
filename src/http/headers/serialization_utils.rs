//! Utility functions for assisting with conversion of headers from and to the HTTP text form.

use std::slice;
use std::ascii::Ascii;
use std::io::IoResult;
use std::strbuf::StrBuf;
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
/// assert_eq!(normalise_header_name(StrBuf::from_str("foo-bar"), "Foo-Bar");
/// assert_eq!(normalise_header_name(StrBuf::from_str("FOO-BAR"), "Foo-Bar");
/// ~~~
pub fn normalise_header_name(name: &StrBuf) -> StrBuf {
    let mut result: StrBuf = StrBuf::with_capacity(name.len());
    let mut capitalise = true;
    for c in name.as_slice().chars() {
        let c = match capitalise {
            true => c.to_ascii().to_upper(),
            false => c.to_ascii().to_lower(),
        };
        result.push_char(c.to_char());
        // ASCII 45 is '-': in that case, capitalise the next char
        capitalise = c.to_byte() == 45;
    }
    result
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
pub fn comma_split(value: &str) -> Vec<StrBuf> {
    value.split(',').map(|w| StrBuf::from_str(w.trim_left())).collect()
}

pub fn comma_split_iter<'a>(value: &'a str)
        -> ::std::iter::Map<'a, &'a str, &'a str, ::std::str::CharSplits<'a, char>> {
    value.split(',').map(|w| w.trim_left())
}

pub trait WriterUtil: Writer {
    fn write_maybe_quoted_string(&mut self, s: &StrBuf) -> IoResult<()> {
        if is_token(s) {
            self.write(s.as_bytes())
        } else {
            self.write_quoted_string(s)
        }
    }

    fn write_quoted_string(&mut self, s: &StrBuf) -> IoResult<()> {
        try!(self.write(['"' as u8]));
        for b in s.as_bytes().iter() {
            if *b == '\\' as u8 || *b == '"' as u8 {
                try!(self.write(['\\' as u8]));
            }
            // XXX This doesn't seem right.
            try!(self.write(&[*b]));
        }
        self.write(['"' as u8])
    }

    fn write_parameter(&mut self, k: &str, v: &StrBuf) -> IoResult<()> {
        try!(self.write(k.as_bytes()));
        try!(self.write(['=' as u8]));
        self.write_maybe_quoted_string(v)
    }

    fn write_parameters(&mut self, parameters: &[(StrBuf, StrBuf)]) -> IoResult<()> {
        for &(ref k, ref v) in parameters.iter() {
            try!(self.write([';' as u8]));
            try!(self.write_parameter(k.as_slice(), v));
        }
        Ok(())
    }

    fn write_quality(&mut self, quality: Option<f64>) -> IoResult<()> {
        // TODO: remove second and third decimal places if zero, and use a better quality type anyway
        match quality {
            Some(qvalue) => write!(&mut *self, ";q={:0.3f}", qvalue),
            None => Ok(()),
        }
    }

    #[inline]
    fn write_token(&mut self, token: &StrBuf) -> IoResult<()> {
        assert!(is_token(token));
        self.write(token.as_bytes())
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
pub fn comma_join(values: &[StrBuf]) -> StrBuf {
    let mut out = StrBuf::new();
    let mut iter = values.iter();
    match iter.next() {
        Some(s) => out.push_str(s.as_slice()),
        None => return out
    }

    for value in iter {
        out.push_str(", ");
        out.push_str(value.as_slice());
    }
    out
}

/// Push a ( token | quoted-string ) onto a string and return it again
pub fn push_maybe_quoted_string(mut s: StrBuf, t: &StrBuf) -> StrBuf {
    if is_token(t) {
        s.push_str(t.as_slice());
        s
    } else {
        push_quoted_string(s, t)
    }
}

/// Make a string into a ( token | quoted-string ), preferring a token
pub fn maybe_quoted_string(s: &StrBuf) -> StrBuf {
    if is_token(s) {
        s.clone()
    } else {
        quoted_string(s)
    }
}

/// Quote a string, to turn it into an RFC 2616 quoted-string
pub fn push_quoted_string(mut s: StrBuf, t: &StrBuf) -> StrBuf {
    let i = s.len();
    s.reserve(t.len() + i + 2);
    s.push_char('"');
    for c in t.as_slice().chars() {
        if c == '\\' || c == '"' {
            s.push_char('\\');
        }
        s.push_char(c);
    }
    s.push_char('"');
    s
}

/// Quote a string, to turn it into an RFC 2616 quoted-string
pub fn quoted_string(s: &StrBuf) -> StrBuf {
    push_quoted_string(StrBuf::new(), s)
}

/// Parse a quoted-string. Returns ``None`` if the string is not a valid quoted-string.
pub fn unquote_string(s: &StrBuf) -> Option<StrBuf> {
    enum State { Start, Normal, Escaping, End }

    let mut state = Start;
    let mut output = StrBuf::new();
    // Strings with escapes cause overallocation, but it's not worth a second pass to avoid this!
    output.reserve(s.len() - 2);
    let mut iter = s.as_slice().chars();
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
pub fn maybe_unquote_string(s: &StrBuf) -> Option<StrBuf> {
    if is_token(s) {
        Some(s.clone())
    } else {
        unquote_string(s)
    }
}

// Takes and emits the StrBuf instead of the &mut str for a simpler, fluid interface
pub fn push_parameter(mut s: StrBuf, k: &StrBuf, v: &StrBuf) -> StrBuf {
    s.push_str(k.as_slice());
    s.push_char('=');
    push_maybe_quoted_string(s, v)
}

// pub fn push_parameters<K: Str, V: Str>(mut s: StrBuf, parameters: &[(K, V)]) -> StrBuf {
pub fn push_parameters(mut s: StrBuf, parameters: &[(StrBuf, StrBuf)]) -> StrBuf {
    for &(ref k, ref v) in parameters.iter() {
        s.push_char(';');
        s = push_parameter(s, k, v);
    }
    s
}

#[cfg(test)]
mod test {
    use super::{normalise_header_name, comma_split, comma_split_iter, comma_join,
                push_parameter, push_parameters, push_maybe_quoted_string, push_quoted_string,
                maybe_quoted_string, quoted_string, unquote_string, maybe_unquote_string};

    #[test]
    #[should_fail]
    fn test_normalise_header_name_fail() {
        normalise_header_name(&StrBuf::from_str("foö-bar"));
    }

    #[test]
    fn test_normalise_header_name() {
        assert_eq!(normalise_header_name(&StrBuf::from_str("foo-bar")), StrBuf::from_str("Foo-Bar"));
        assert_eq!(normalise_header_name(&StrBuf::from_str("FOO-BAR")), StrBuf::from_str("Foo-Bar"));
    }

    #[test]
    fn test_comma_split() {
        // Simple 0-element case
        assert_eq!(comma_split(""), vec!(StrBuf::new()));
        // Simple 1-element case
        assert_eq!(comma_split("foo"), vec!(StrBuf::from_str("foo")));
        // Simple 2-element case
        assert_eq!(comma_split("foo,bar"), vec!(StrBuf::from_str("foo"), StrBuf::from_str("bar")));
        // Simple >2-element case
        assert_eq!(comma_split("foo,bar,baz,quux"), vec!(StrBuf::from_str("foo"), StrBuf::from_str("bar"), StrBuf::from_str("baz"), StrBuf::from_str("quux")));
        // Doesn't handle quoted-string intelligently
        assert_eq!(comma_split("\"foo,bar\",baz"), vec!(StrBuf::from_str("\"foo"), StrBuf::from_str("bar\""), StrBuf::from_str("baz")));
        // Doesn't do right trimming, but does left
        assert_eq!(comma_split(" foo;q=0.8 , bar/* "), vec!(StrBuf::from_str("foo;q=0.8 "), StrBuf::from_str("bar/* ")));
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
        assert_eq!(comma_join([StrBuf::new()]), StrBuf::new());
        assert_eq!(comma_join([StrBuf::from_str("foo")]), StrBuf::from_str("foo"));
        assert_eq!(comma_join([StrBuf::from_str("foo"), StrBuf::from_str("bar")]), StrBuf::from_str("foo, bar"));
        assert_eq!(comma_join([StrBuf::from_str("foo"), StrBuf::from_str("bar"), StrBuf::from_str("baz"), StrBuf::from_str("quux")]), StrBuf::from_str("foo, bar, baz, quux"));
        assert_eq!(comma_join([StrBuf::from_str("\"foo,bar\""), StrBuf::from_str("baz")]), StrBuf::from_str("\"foo,bar\", baz"));
        assert_eq!(comma_join([StrBuf::from_str(" foo;q=0.8 "), StrBuf::from_str("bar/* ")]), StrBuf::from_str(" foo;q=0.8 , bar/* "));
    }

    #[test]
    fn test_push_maybe_quoted_string() {
        assert_eq!(push_maybe_quoted_string(StrBuf::from_str("foo,"), &StrBuf::from_str("bar")), StrBuf::from_str("foo,bar"));
        assert_eq!(push_maybe_quoted_string(StrBuf::from_str("foo,"), &StrBuf::from_str("bar/baz")), StrBuf::from_str("foo,\"bar/baz\""));
    }

    #[test]
    fn test_maybe_quoted_string() {
        assert_eq!(maybe_quoted_string(&StrBuf::from_str("bar")), StrBuf::from_str("bar"));
        assert_eq!(maybe_quoted_string(&StrBuf::from_str("bar/baz \"yay\"")), StrBuf::from_str("\"bar/baz \\\"yay\\\"\""));
    }

    #[test]
    fn test_push_quoted_string() {
        assert_eq!(push_quoted_string(StrBuf::from_str("foo,"), &StrBuf::from_str("bar")), StrBuf::from_str("foo,\"bar\""));
        assert_eq!(push_quoted_string(StrBuf::from_str("foo,"), &StrBuf::from_str("bar/baz \"yay\\\"")),
                   StrBuf::from_str("foo,\"bar/baz \\\"yay\\\\\\\"\""));
    }

    #[test]
    fn test_quoted_string() {
        assert_eq!(quoted_string(&StrBuf::from_str("bar")), StrBuf::from_str("\"bar\""));
        assert_eq!(quoted_string(&StrBuf::from_str("bar/baz \"yay\\\"")), StrBuf::from_str("\"bar/baz \\\"yay\\\\\\\"\""));
    }

    #[test]
    fn test_unquote_string() {
        assert_eq!(unquote_string(&StrBuf::from_str("bar")), None);
        assert_eq!(unquote_string(&StrBuf::from_str("\"bar\"")), Some(StrBuf::from_str("bar")));
        assert_eq!(unquote_string(&StrBuf::from_str("\"bar/baz \\\"yay\\\\\\\"\"")), Some(StrBuf::from_str("bar/baz \"yay\\\"")));
        assert_eq!(unquote_string(&StrBuf::from_str("\"bar")), None);
        assert_eq!(unquote_string(&StrBuf::from_str(" \"bar\"")), None);
        assert_eq!(unquote_string(&StrBuf::from_str("\"bar\" ")), None);
        assert_eq!(unquote_string(&StrBuf::from_str("\"bar\" \"baz\"")), None);
        assert_eq!(unquote_string(&StrBuf::from_str("\"bar/baz \\\"yay\\\\\"\"")), None);
    }

    #[test]
    fn test_maybe_unquote_string() {
        assert_eq!(maybe_unquote_string(&StrBuf::from_str("bar")), Some(StrBuf::from_str("bar")));
        assert_eq!(maybe_unquote_string(&StrBuf::from_str("\"bar\"")), Some(StrBuf::from_str("bar")));
        assert_eq!(maybe_unquote_string(&StrBuf::from_str("\"bar/baz \\\"yay\\\\\\\"\"")), Some(StrBuf::from_str("bar/baz \"yay\\\"")));
        assert_eq!(maybe_unquote_string(&StrBuf::from_str("\"bar")), None);
        assert_eq!(maybe_unquote_string(&StrBuf::from_str(" \"bar\"")), None);
        assert_eq!(maybe_unquote_string(&StrBuf::from_str("\"bar\" ")), None);
        assert_eq!(maybe_unquote_string(&StrBuf::from_str("\"bar\" \"baz\"")), None);
        assert_eq!(maybe_unquote_string(&StrBuf::from_str("\"bar/baz \\\"yay\\\\\"\"")), None);
    }

    #[test]
    fn test_push_parameter() {
        assert_eq!(push_parameter(StrBuf::from_str("foo"), &StrBuf::from_str("bar"), &StrBuf::from_str("baz")), StrBuf::from_str("foobar=baz"));
        assert_eq!(push_parameter(StrBuf::from_str("foo"), &StrBuf::from_str("bar"), &StrBuf::from_str("baz/quux")), StrBuf::from_str("foobar=\"baz/quux\""));
    }

    #[test]
    fn test_push_parameters() {
        assert_eq!(push_parameters(StrBuf::from_str("foo"), []), StrBuf::from_str("foo"));
        assert_eq!(push_parameters(StrBuf::from_str("foo"), [(StrBuf::from_str("bar"), StrBuf::from_str("baz"))]), StrBuf::from_str("foo;bar=baz"));
        assert_eq!(push_parameters(StrBuf::from_str("foo"), [(StrBuf::from_str("bar"), StrBuf::from_str("baz/quux"))]), StrBuf::from_str("foo;bar=\"baz/quux\""));
        assert_eq!(push_parameters(StrBuf::from_str("foo"), [(StrBuf::from_str("bar"), StrBuf::from_str("baz")), (StrBuf::from_str("quux"), StrBuf::from_str("fuzz"))]),
                   StrBuf::from_str("foo;bar=baz;quux=fuzz"));
        assert_eq!(push_parameters(StrBuf::from_str("foo"), [(StrBuf::from_str("bar"), StrBuf::from_str("baz")), (StrBuf::from_str("quux"), StrBuf::from_str("fuzz zee"))]),
                   StrBuf::from_str("foo;bar=baz;quux=\"fuzz zee\""));
        assert_eq!(push_parameters(StrBuf::from_str("foo"), [(StrBuf::from_str("bar"), StrBuf::from_str("baz/quux")), (StrBuf::from_str("fuzz"), StrBuf::from_str("zee"))]),
                   StrBuf::from_str("foo;bar=\"baz/quux\";fuzz=zee"));
    }
}
