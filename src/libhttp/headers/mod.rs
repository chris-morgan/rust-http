//! Types and utilities for working with headers in HTTP requests and responses.
//!
//! Notably, HTTP headers are case insensitive; headers are represented as a `TreeMap`, and so there
//! is, at present, scope for mistakes in providing multiple values for a header under different
//! cases. You should, of course, avoid this.
//!
//! Headers can be normalised into the canonical case employed in this library with
//! `normalise_header_name`; that function defines the canonical case. Notably, this will mean that
//! the naming conventions are *not* followed for certain headers: `Content-MD5` would be sent as
//! `Content-Md5`, `TE` as `Te` and `WWW-Authenticate` as `Www-Authenticate`.
//!
//! Another common convention among HTTP headers is to use comma-separated values,
//! e.g. `Accept: text/html, text/plain;q=0.8, text/*;q=0.1`. For transforming to and from these
//! values we have `comma_split` and `comma_join`.

use std::rt::io::{Reader, Stream};
use extra::treemap::TreeMap;
use extra::time::{Tm, strptime};
use rfc2616::{is_token_item, CR, LF, SP, HT, COLON, DOUBLE_QUOTE, BACKSLASH};
use method::Method;

use self::serialization_utils::{normalise_header_name, push_quality};
use self::serialization_utils::{push_maybe_quoted_string, maybe_quoted_string, push_quoted_string};
use self::serialization_utils::{push_key_value_pair, push_key_value_pairs};
use self::serialization_utils::{unquote_string, maybe_unquote_string};
use self::serialization_utils::{parameter_split, comma_split, comma_split_iter, comma_join};

mod request;
mod response;
mod serialization_utils;

pub type Headers = TreeMap<~str, ~str>;

/*impl Headers {
    fn new() -> ~Headers {
        ~Headers(*TreeMap::new::<~str, ~str>())
    }
}*/

/* TODO: ensure we've got all standard HTTP headers, not just those in RFC 2616.

From https://en.wikipedia.org/wiki/List_of_HTTP_headers:

- Known missing headers:

  - Access-Control-Allow-Origin
  - Content-Disposition
  - Link
  - P3P
  - Refresh
  - Set-Cookie
  - Status
  - Strict-Transport-Security

- Common non-standard headers:

  - X-Frame-Options
  - X-XSS-Protection
  - Content-Security-Policy, X-Content-Security-Policy, X-WebKit-CSP
  - X-Content-Type-Options
  - X-Powered-By
  - X-UA-Compatible

Also go through http://www.iana.org/assignments/message-headers/message-headers.xhtml which should
be the more canonical source.

*/

//pub mod accept;
//pub mod accept_charset;
//pub mod accept_encoding;
//pub mod accept_language;
pub mod accept_ranges;
pub mod allow;
//pub mod cache_control;
pub mod connection;
//pub mod content_encoding;
//pub mod content_range;
//pub mod content_type;
//pub mod etag;
pub mod host;

pub type DeltaSeconds = u64;

pub trait HeaderEnum {
    fn header_name<'a>(&'a self) -> &'a str;

    fn write_header<T: Writer>(&self, writer: &mut T);

    // FIXME: this method combination is temporary, to be merged with an efficient parser like that
    // of the request line. Also refactor to remove the need to return the next byte too.
    /// Return values:
    ///
    /// - Header
    /// - Next byte (consumed out of linear white space checking necessity)
    ///
    /// (None, None) means EOF.
    /// (None, Some) means malformed header.
    /// (Some, None) is impossible.
    /// (Some, Some) means you have a valid header.
    fn from_stream<T: Reader>(reader: &mut T) -> (Option<Self>, Option<u8>) {
        let mut name_finished = false;
        let mut header_name = ~"";
        loop {
            match reader.read_byte() {
                Some(b) if !name_finished && is_token_item(b) => header_name.push_char(b as char),
                // TODO: check up on the rules for a line like "Name : value". Full LWS?
                Some(b) if b == SP => name_finished = true,
                Some(b) if b == COLON => break,
                _ => return (None, None),
            }
        }
        let mut iter = HeaderValueByteIterator {
            reader: reader,
            next_byte: None,
            state: Normal,
        };
        header_name = normalise_header_name(header_name);
        let header = HeaderEnum::value_from_stream(header_name, &mut iter);
        // Ensure that the entire header line is consumed (don't want to mess up next header!)
        for _ in iter { }
        (header, iter.next_byte)
    }

    fn value_from_stream<T: Reader>(name: ~str, input: &mut HeaderValueByteIterator<T>)
        -> Option<Self>;
}

#[deriving(Eq)]
enum HeaderValueByteIteratorState {
    Normal,  // Anything other than the rest.
    InsideQuotedString,  // no LWS compacting here. TODO: check spec on CR LF SP in quoted-string.
    InsideQuotedStringEscape,  // don't let a " close quoted-string if it comes next
    CompactingLWS,  // Got at least one linear white space character; turning it into just one SP
    GotCR,  // Last character was CR
    GotCRLF,  // Last two characters were CR LF
    Finished,  // Finished, so next() should always return ``None`` immediately (no side effects)
}

/// An iterator over the bytes of a header value.
/// This ensures one cannot read past the end of a header mistakenly and that linear white space is
/// handled correctly so that nothing else needs to worry about it. Any linear whitespace (multiple
/// spaces outside of a quoted-string) is compacted into a single SP.
pub struct HeaderValueByteIterator<'self, R> {
    reader: &'self mut R,

    /// This field serves two purposes. *During* iteration, it will typically be ``None``, but
    /// certain cases will cause it to be a ``Some``, meaning that the next ``next()`` call will
    /// return that value rather than reading a new byte. At the *end* of iteration (after
    /// ``next()`` has returned ``None``), it will be the extra byte which it has had to consume
    /// from the stream because of the possibility of linear white space of the form ``CR LF SP``.
    /// It is guaranteed that if ``self.state == Finished`` this will be a ``Some``.
    next_byte: Option<u8>,

    state: HeaderValueByteIteratorState,
}

impl<'self, R: Reader> HeaderValueByteIterator<'self, R> {
    // TODO: can we have collect() implemented for ~str? That would negate the need for this.
    fn collect_to_str(&mut self) -> ~str {
        // TODO: be more efficient (char cast is a little unnecessary)
        let out = ~"";
        // No point in trying out.reserve(self.size_hint()); I *know* I can't offer a useful hint.
        for b in self {
            out.push_char(b as char);
        }
        out
    }
}

impl<'self, R: Reader> Iterator<u8> for HeaderValueByteIterator<'self, R> {
    fn next(&mut self) -> Option<u8> {
        if self.state == Finished {
            return None;
        }
        let b = match self.next_byte {
            Some(b) => {
                self.next_byte = None;
                b
            },
            None => match self.reader.read_byte() {
                None => {
                    // EOF; not a friendly reader :-(. Let's just call that the end.
                    self.state = Finished;
                    return None
                },
                Some(b) => b,
            }
        };
        self.state = match self.state {
            Normal | CompactingLWS if b == CR => {
                // It's OK to go to GotCR on CompactingLWS: if it ends up ``CR LF SP`` it'll get
                // back to CompactingLWS, and if it ends up ``CR LF`` we didn't need the
                // trailing whitespace anyway.
                GotCR
            },

            // TODO: fix up these quoted-string rules, they're probably wrong (CRLF inside it?)
            Normal if b == DOUBLE_QUOTE => InsideQuotedString,
            InsideQuotedString if b == BACKSLASH => InsideQuotedStringEscape,
            InsideQuotedStringEscape => InsideQuotedString,
            InsideQuotedString if b == DOUBLE_QUOTE => Normal,

            GotCR | Normal if b == LF => {
                // TODO: check RFC 2616's precise rules, I think it does say that a server
                // should also accept missing CR
                GotCRLF
            },
            GotCR => {
                // False alarm, CR without LF. Hmm... was it LWS then? TODO.
                // FIXME: this is BAD, but I'm dropping the CR for now;
                // when we can have yield it'd be better. Also again, check speck.
                self.next_byte = Some(b);
                return Some(CR);
            },
            GotCRLF if b == SP || b == HT => {
                // CR LF SP is a suitable linear whitespace, so don't stop yet
                CompactingLWS
            },
            GotCRLF => {
                // Ooh! We got to a genuine end of line, so we're done.
                // But first, we must makes sure not to lose that byte.
                self.final_byte = Some(b);
                self.state = Finished;
                return None;
            },
            Normal | CompactingLWS if b == SP || b == HT => {
                // Start or continue to compact linear whitespace
                CompactingLWS
            },
            CompactingLWS => {
                // End of LWS, compact it down to a single space, unless it's at the start.
                if !self.at_start {
                    self.next_byte = Some(b);
                }
                self.state = Normal;
                return Some(SP);
            },
            Normal => {
                Normal
            },
        };
        Some(b)
    }
}

/**
 * A datatype for headers.
 */
pub trait HeaderConvertible {
    /**
     * Read a header value from an iterator over the raw value. That iterator compacts linear white
     * space to a single SP, so this static method should just expect a single SP. There will be no
     * leading or trailing space, either; also the ``CR LF`` which would indicate the end of the
     * header line in the stream is removed.
     *
     * For types that implement ``FromStr``, a sane-but-potentially-not-as-fast-as-possible default
     * would be::
     *
     *     FromStr::from_str(reader.collect_to_str())
     *
     * (This is not provided as a default implementation as owing to present upstream limitations
     * that would require the type to implement FromStr also, which is not considered reasonable.)
     */
    fn from_stream<T: Reader>(reader: &mut HeaderValueByteIterator<T>) -> Option<Self>;

    /**
     * Write the HTTP value of the header to the stream.
     *
     * The default implementation uses the ``http_value`` method; for now, this should tend to be
     * enough, but there may be more efficient ways to do it in certain cases.
     */
    fn to_stream<T: Writer>(&self, writer: &mut T) {
        let s = self.http_value();
        writer.write(s.as_bytes());
    }

    /**
     * The value of the header as it would be written for an HTTP header.
     *
     * For types which implement ``ToStr``, a body of ``self.to_str()`` will often be sufficient.
     */
    fn http_value(&self) -> ~str;
}

// Now let's have some common implementation types.
// Some header types really are arbitrary strings. Let's cover that case here.
impl HeaderConvertible for ~str {
    fn from_stream<T: Reader>(reader: &mut HeaderValueByteIterator<T>) -> Option<~str> {
        Some(reader.collect_to_str())
    }

    fn to_stream<T: Writer>(&self, writer: &mut T) {
        writer.write(self.as_bytes());
    }

    fn http_value(&self) -> ~str {
        self.to_owned()
    }
}

/**
 * ``HTTP-date`` is represented as a ``Tm``. (What follows is a quotation from RFC 2616.)
 *
 * HTTP applications have historically allowed three different formats
 * for the representation of date/time stamps:
 *
 *    Sun, 06 Nov 1994 08:49:37 GMT  ; RFC 822, updated by RFC 1123
 *    Sunday, 06-Nov-94 08:49:37 GMT ; RFC 850, obsoleted by RFC 1036
 *    Sun Nov  6 08:49:37 1994       ; ANSI C's asctime() format
 *
 * The first format is preferred as an Internet standard and represents
 * a fixed-length subset of that defined by RFC 1123 [8] (an update to
 * RFC 822 [9]). The second format is in common use, but is based on the
 * obsolete RFC 850 [12] date format and lacks a four-digit year.
 * HTTP/1.1 clients and servers that parse the date value MUST accept
 * all three formats (for compatibility with HTTP/1.0), though they MUST
 * only generate the RFC 1123 format for representing HTTP-date values
 * in header fields. See section 19.3 for further information.
 *
 *    Note: Recipients of date values are encouraged to be robust in
 *    accepting date values that may have been sent by non-HTTP
 *    applications, as is sometimes the case when retrieving or posting
 *    messages via proxies/gateways to SMTP or NNTP.
 *
 * All HTTP date/time stamps MUST be represented in Greenwich Mean Time
 * (GMT), without exception. For the purposes of HTTP, GMT is exactly
 * equal to UTC (Coordinated Universal Time). This is indicated in the
 * first two formats by the inclusion of "GMT" as the three-letter
 * abbreviation for time zone, and MUST be assumed when reading the
 * asctime format. HTTP-date is case sensitive and MUST NOT include
 * additional LWS beyond that specifically included as SP in the
 * grammar.
 *
 *     HTTP-date    = rfc1123-date | rfc850-date | asctime-date
 *     rfc1123-date = wkday "," SP date1 SP time SP "GMT"
 *     rfc850-date  = weekday "," SP date2 SP time SP "GMT"
 *     asctime-date = wkday SP date3 SP time SP 4DIGIT
 *     date1        = 2DIGIT SP month SP 4DIGIT
 *                    ; day month year (e.g., 02 Jun 1982)
 *     date2        = 2DIGIT "-" month "-" 2DIGIT
 *                    ; day-month-year (e.g., 02-Jun-82)
 *     date3        = month SP ( 2DIGIT | ( SP 1DIGIT ))
 *                    ; month day (e.g., Jun  2)
 *     time         = 2DIGIT ":" 2DIGIT ":" 2DIGIT
 *                    ; 00:00:00 - 23:59:59
 *     wkday        = "Mon" | "Tue" | "Wed"
 *                  | "Thu" | "Fri" | "Sat" | "Sun"
 *     weekday      = "Monday" | "Tuesday" | "Wednesday"
 *                  | "Thursday" | "Friday" | "Saturday" | "Sunday"
 *     month        = "Jan" | "Feb" | "Mar" | "Apr"
 *                  | "May" | "Jun" | "Jul" | "Aug"
 *                  | "Sep" | "Oct" | "Nov" | "Dec"
 *
 *    Note: HTTP requirements for the date/time stamp format apply only
 *    to their usage within the protocol stream. Clients and servers are
 *    not required to use these formats for user presentation, request
 *    logging, etc.
 */
impl HeaderConvertible for Tm {
    fn from_stream<T: Reader>(reader: &mut HeaderValueByteIterator<T>) -> Option<Tm> {
        let value = reader.collect_to_str();

        // XXX: %Z actually ignores any timezone other than UTC. Probably not a good idea?
        match strptime(value, "%a, %d %b %Y %T %Z") {  // RFC 822, updated by RFC 1123
            Ok(time) => return Some(time),
            Err(*) => ()
        }

        match strptime(value, "%A, %d-%b-%y %T %Z") {  // RFC 850, obsoleted by RFC 1036
            Ok(time) => return Some(time),
            Err(*) => ()
        }

        match strptime(value, "%c") {  // ANSI C's asctime() format
            Ok(time) => Some(time),
            Err(*) => None
        }
    }

    fn to_stream<T: Writer>(&self, writer: &mut T) {
        writer.write(self.to_utc().strftime("%a, %d %b %Y %T GMT").as_bytes());
    }

    fn http_value(&self) -> ~str {
        self.to_utc().strftime("%a, %d %b %Y %T GMT")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use headers::test_utils::{from_stream_with_str, to_stream_into_str};

    fn sample_tm(zone: ~str) -> Option<Date> {
        Ok(Date {
            tm_sec: 37,
            tm_min: 49,
            tm_hour: 8,
            tm_mday: 6,
            tm_mon: 10,
            tm_year: 94,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: zone,
            tm_nsec: 0
        })
    }

    /// Test `from_stream` with an RFC 822 time (updated by RFC 1123)
    #[test]
    fn test_from_stream_rfc822() {
        assert_eq!(from_stream_with_str("Sun, 06 Nov 1994 08:49:37 GMT"), sample_tm(~"UTC"));
    }

    /// Test `from_stream` with an RFC 850 time (obsoleted by RFC 1036)
    #[test]
    fn test_from_stream_rfc850() {
        assert_eq!(from_stream_with_str("Sunday, 06-Nov-94 08:49:37 GMT"), sample_tm(~"UTC"));
    }

    /// Test `from_stream` with the ANSI C's asctime() format
    #[test]
    fn test_from_stream_asctime() {
        assert_eq!(from_stream_with_str("Sun Nov  6 08:49:37 1994"), sample_tm(~""));
    }

    /// Test `http_value`, which outputs an RFC 1123 time
    #[test]
    fn test_http_value() {
        assert_eq!(sample_tm(~"UTC").http_value(), ~"Sun, 06 Nov 1994 08:49:37 GMT");
    }

    /// Test `to_stream`, which outputs an RFC 1123 time
    #[test]
    fn test_to_stream() {
        assert_eq!(to_stream_into_str(sample_tm(~"UTC")), ~"Sun, 06 Nov 1994 08:49:37 GMT");
    }
}
