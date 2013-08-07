//! The old, traditional way of reading things: load it and then use it, rather than bit by bit.
//! We'll see which way ends up better.

use super::super::method::{Method};
use super::super::status;
use std::{str, uint};
use super::super::rfc2616::{CR, LF};
use super::{RequestBuffer, RequestUri};

/// Should be exactly equivalent in functionality to RequestBuffer.read_request_line.
pub fn read_request_line(self_: &mut RequestBuffer)
        -> Result<(Method, RequestUri, (uint, uint)), status::Status> {
    parse_request_line(read_crlf_line(self_))
}

/// Read a line ending in CRLF
fn read_crlf_line(self_: &mut RequestBuffer) -> ~str {
    self_.line_bytes.clear();

    enum State { Normal, GotCR };
    let mut state = Normal;

    loop {
        state = match self_.stream.read_byte() {
            // Client closed connection (e.g. keep-alive timeout, connect without HTTP request):
            None => fail!("EOF"),
            Some(b) => match state {
                Normal if b == CR => {
                    GotCR
                },
                GotCR if b == LF => {
                    break;
                },
                GotCR => {
                    self_.line_bytes.push(CR);
                    self_.line_bytes.push(b);
                    Normal
                },
                Normal => {
                    self_.line_bytes.push(b);
                    Normal
                }
            }
        };
    }
    str::from_bytes(self_.line_bytes)
}

/// Parse an HTTP request line into its parts.
///
/// `parse_request_line("GET /foo HTTP/1.1") == Ok((method::Get, AbsolutePath("/foo"), (1, 1)))`
fn parse_request_line(line: &str) -> Result<(Method, RequestUri, (uint, uint)), status::Status> {
    let mut words = line.word_iter();
    let method = match words.next() {
        Some(s) => Method::from_str_or_new(s),
        None => return Err(status::BadRequest),
    };
    let request_uri = match words.next() {
        Some(s) => match FromStr::from_str::<RequestUri>(s) {
            Some(r) => r,
            None => return Err(status::BadRequest),
        },
        None => return Err(status::BadRequest),
    };
    let http_version = match words.next() {
        Some(s) => parse_http_version(s),
        None => return Err(status::BadRequest),
    };
    match (words.next(), http_version) {
        (None, Some(v)) => Ok((method, request_uri, v)),
        _ => Err(status::BadRequest),  // More words or invalid HTTP version
    }
}

/// Parse an HTTP version string into the two X.Y parts.
///
/// At present, HTTP versions the server does not know about are rejected.
///
/// ~~~ {.rust}
/// assert_eq!(parse_http_version(~"HTTP/1.0"), Some((1, 0)))
/// assert_eq!(parse_http_version(~"HTTP/1.1"), Some((1, 1)))
/// assert_eq!(parse_http_version(~"HTTP/2.0"), Some((2, 0)))
/// ~~~
fn parse_http_version(version: &str) -> Option<(uint, uint)> {
    match version.to_ascii().to_upper().to_str_ascii() {
        // These two are efficiency shortcuts; they're expected to be all that we ever receive,
        // but naturally we mustn't let it crash on other inputs.
        ~"HTTP/1.0" => Some((1, 0)),
        ~"HTTP/1.1" => Some((1, 1)),
        ref v if v.starts_with("HTTP/") => {
            // This commented-out variant would fail! if given non-integers
            //let ints: ~[uint] = v.slice_from(5).split_iter('.').map(
            //    |&num| uint::from_str_radix(num, 10).unwrap()).collect();
            let mut ints = [0u, 0u];
            for (i, strnum) in v.slice_from(5).split_iter('.').enumerate() {
                if i > 1 {
                    // More than two numbers, e.g. HTTP/1.2.3
                    return None;
                }
                match uint::from_str_radix(strnum, 10) {
                    Some(num) => ints[i] = num,
                    None => return None,
                }
            }
            Some((ints[0], ints[1]))
        }
        _ => None
    }
}

// Now why should these go in a separate module? Especially when we're testing private methods...
#[test]
fn test_parse_http_version() {
    assert_eq!(parse_http_version("http/1.1"), Some((1, 1)));
    assert_eq!(parse_http_version("hTTp/1.1"), Some((1, 1)));
    assert_eq!(parse_http_version("HTTP/1.1"), Some((1, 1)));
    assert_eq!(parse_http_version("HTTP/1.0"), Some((1, 0)));
    assert_eq!(parse_http_version("HTTP/2.34"), Some((2, 34)));
}

#[test]
fn test_parse_request_line() {
    use super::method::{Get, Post};
    assert_eq!(parse_request_line("GET /foo HTTP/1.1"),
        Some((Get, AbsolutePath(~"/foo"), (1, 1))));
    assert_eq!(parse_request_line("POST / http/1.0"),
        Some((Post, AbsolutePath(~"/"), (1, 0))));
    assert_eq!(parse_request_line("POST / HTTP/2.0"),
        Some((Post, AbsolutePath(~"/"), (2, 0))));
}
