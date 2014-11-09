//! Types and utilities for working with headers in HTTP requests and responses.
//!
//! This HTTP system is Special in that it uses lots of strong typing for its header system. All
//! known HTTP headers are type checked, rather than being dealt with as strings all the time. Only
//! unknown headers are stored in a map in the traditional way.

use url::Url;
use std::io::IoResult;
use time::{Tm, strptime};
use rfc2616::{is_token_item, is_separator, CR, LF, SP, HT, COLON};
use method::Method;

use self::serialization_utils::{normalise_header_name};

pub enum HeaderLineErr { EndOfFile, EndOfHeaders, MalformedHeaderValue, MalformedHeaderSyntax }

pub mod test_utils;
pub mod serialization_utils;

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
//pub mod cache_control;
pub mod connection;
//pub mod content_encoding;
//pub mod content_range;
pub mod content_type;
pub mod etag;
pub mod host;
pub mod transfer_encoding;

pub type DeltaSeconds = u64;

#[deriving(Clone, PartialEq, Eq)]
pub enum ConsumeCommaLWSResult {
    CommaConsumed,
    EndOfValue,
    ErrCommaNotFound,
}

pub trait HeaderEnum {
    fn header_name(&self) -> String;
    fn header_value(&self) -> String;
    fn write_header<W: Writer>(&self, writer: &mut W) -> IoResult<()>;

    // FIXME: this method combination is temporary, to be merged with an efficient parser like that
    // of the request line. Also refactor to remove the need to return the next byte too.
    /// Return values:
    ///
    /// - Header
    /// - Next byte (consumed out of linear white space checking necessity)
    ///
    /// (Err, Option) means EOF/EOH/Malformed.
    /// (Ok, None) is impossible.
    /// (Ok, Some) means you have a valid header.
    //REMOVED BECAUSE OF ICE:
    //fn from_stream<T: Reader>(reader: &mut T) -> (Result<Self, HeaderLineErr>, Option<u8>) { ... }

    fn value_from_stream<R: Reader>(name: String, input: &mut HeaderValueByteIterator<R>)
        -> Option<Self>;
}

/// Shifted out of being a default method to fix an ICE (not yet reported, TODO)
pub fn header_enum_from_stream<R: Reader, E: HeaderEnum>(reader: &mut R)
        -> (Result<E, HeaderLineErr>, Option<u8>) {
    enum State { Start, ReadingName, NameFinished, GotCR }
    let mut state = Start;
    let mut header_name = String::new();
    loop {
        state = match (state, reader.read_byte()) {
            (Start, Ok(b)) | (ReadingName, Ok(b)) if is_token_item(b) => {
                header_name.push(b as char);
                ReadingName
            },
            // TODO: check up on the rules for a line like "Name : value". Full LWS?
            (Start, Ok(b)) if b == CR => GotCR,
            (Start, Ok(b)) | (GotCR, Ok(b)) if b == LF => {
                return (Err(EndOfHeaders), None);
            },
            (_, Ok(b)) if b == SP => NameFinished,
            (_, Ok(b)) if b == COLON => break,
            (_, Ok(_)) => return (Err(MalformedHeaderSyntax), None),
            (_, Err(_)) => return (Err(EndOfFile), None),
        }
    }
    let mut iter = HeaderValueByteIterator::new(reader);
    let header = HeaderEnum::value_from_stream(normalise_header_name(&header_name), &mut iter);
    // Ensure that the entire header line is consumed (don't want to mess up next header!)
    for _ in iter { }
    match header {
        Some(h) => (Ok(h), iter.next_byte),
        None => {
            debug!("malformed header value for {}", header_name[]);
            // Alas, I can't tell you what the value actually was... TODO: improve that situation
            (Err(MalformedHeaderValue), iter.next_byte)
        },
    }
}

#[deriving(PartialEq, Eq)]
enum HeaderValueByteIteratorState {
    Normal,  // Anything other than the rest.
    GotLF,  // Last character was LF (could be end of header or, if followed by SP or HT, LWS)
    Finished,  // Finished, so next() should always return ``None`` immediately (no side effects)
}

/// An iterator over the bytes of a header value.
/// This ensures one cannot read past the end of a header mistakenly and that linear white space is
/// handled correctly so that nothing else needs to worry about it. Any linear whitespace (multiple
/// spaces outside of a quoted-string) is compacted into a single SP.
pub struct HeaderValueByteIterator<'a, R: 'a> {
    pub reader: &'a mut R,

    /// This field serves two purposes. *During* iteration, it will typically be ``None``, but
    /// certain cases will cause it to be a ``Some``, meaning that the next ``next()`` call will
    /// return that value rather than reading a new byte. At the *end* of iteration (after
    /// ``next()`` has returned ``None``), it will be the extra byte which it has had to consume
    /// from the stream because of the possibility of linear white space of the form ``CR LF SP``.
    /// It is guaranteed that if ``self.state == Finished`` this will be a ``Some``.
    pub next_byte: Option<u8>,

    pub at_start: bool,
    state: HeaderValueByteIteratorState,
}

impl<'a, R: Reader> HeaderValueByteIterator<'a, R> {

    pub fn new(reader: &'a mut R) -> HeaderValueByteIterator<'a, R> {
        HeaderValueByteIterator {
            reader: reader,
            next_byte: None,
            at_start: true,
            state: Normal,
        }
    }

    /// Check that the entire header value has been consumed.
    ///
    /// Should there be any trailing linear white space, it is dropped.
    ///
    /// Be cautious using this function as it is destructive, losing a character in the case where
    /// the value has not been entirely consumed.
    ///
    /// This should only be called when finished with a value and ensuring that there aren't
    /// unexpected characters
    ///
    /// Suggested usage is in a ``from_stream`` method::
    ///
    ///```ignore
    ///if reader.verify_consumed() {
    ///    Some(header)
    ///} else {
    ///    None
    ///}
    ///```
    ///
    /// ... however, this common case is handled with the ``some_if_consumed`` method, so you may
    /// very well not need to call this function directly.
    #[inline]
    pub fn verify_consumed(&mut self) -> bool {
        self.consume_optional_lws();
        self.next() == None
    }

    /// Turn a constructed header value into an Option: Some(value) if the header value is consumed
    /// or None if it is not, thus indicating: "I'm finished and expect nothing more. Anything more
    /// is an error."
    #[inline]
    pub fn some_if_consumed<T>(&mut self, t: T) -> Option<T> {
        if self.verify_consumed() {
            Some(t)
        } else {
            None
        }
    }

    // TODO: can we have collect() implemented for String? That would negate the need for this.
    fn collect_to_string(&mut self) -> String {
        // TODO: be more efficient (char cast is a little unnecessary)
        let mut out = String::new();
        // No point in trying out.reserve(self.size_hint()); I *know* I can't offer a useful hint.
        loop {
            match self.next() {
                None => break,
                Some(b) => out.push(b as char),
            }
        }
        /*Doesn't work: "cannot borrow immutable self value as mutable" (!! TODO: report bug)
        for b in self {
            out.push(b as char);
        }*/
        out
    }

    /**
     * Consume optional `*LWS`. That is, zero or more of SP and HT, until it
     * gets to something other than SP and HT or gets to the end of the header.
     */
    pub fn consume_optional_lws(&mut self) {
        // This is the dirty secret of this method.
        let _ = self.consume_lws();
    }

    /**
     * Consume `1*LWS`. That is, one or more of SP and HT, until it gets to
     * something other than SP and HT or gets to the end of the header.
     */
    pub fn consume_lws(&mut self) -> bool {
        // This doesn't need to deal with CR and LF; next() collapses that LWS.
        // (This Serious Comment had to come here rather than lower where it would normally be more
        // suitable as it would otherwise have messed up the poetry of the Less Serious Comments.)

        // Don't you love descriptive variable names?
        let mut the_savages_gobbled_up_at_least_one_white_space_char = false;
        loop {
            // 1000 hairy (?) white space chars, sitting in a stream...
            match self.next() {
                // Gobble, gobble, glup, glup, much, munch, munch.
                Some(b) if b == SP || b == HT => (),
                Some(b) => {
                    // "Oy! Who put a non-white-space-char on my plate?"
                    // Better take it off or someone might eat it by accident...
                    assert_eq!(self.next_byte, None);  // TODO: manually verify this holds
                    self.next_byte = Some(b);
                    break;
                },
                None => break,  // Sorry, you're going to go hungry today...
            };
            the_savages_gobbled_up_at_least_one_white_space_char = true;
        }
        the_savages_gobbled_up_at_least_one_white_space_char
    }

    /// Return values:
    /// - CommaConsumed if there was a comma and it was consumed;
    /// - EndOfValue if the header value has been completely consumed;
    /// - ErrCommaNotFound if the next thing wasn't a comma (this is an error state)
    pub fn consume_comma_lws(&mut self) -> ConsumeCommaLWSResult {
        self.consume_optional_lws();
        match self.next() {
            Some(b',') => {
                self.consume_optional_lws();
                CommaConsumed
            },
            Some(_) => {
                ErrCommaNotFound
            },
            None => {
                EndOfValue
            }
        }
    }

    /// Read a quoted-string from the current position.
    /// If the quoted-string is not begun immediately or the header ends before it is completed,
    /// then None is returned; TODO: decide if I can return the bytes read (at present, escapes and
    /// double quote would be lost if I did that).
    pub fn read_quoted_string(&mut self, already_opened: bool) -> Option<String> {
        enum State { Start, Normal, Escaping }

        let mut state = if already_opened { Normal } else { Start };
        let mut output = String::new();
        loop {
            match self.next() {
                None => return None,
                Some(b) => {
                    state = match state {
                        Start if b == b'"' => Normal,
                        Start => return None,
                        Normal if b == b'\\' => Escaping,
                        Normal if b == b'"' => break,
                        Normal | Escaping => { output.push(b as char); Normal },
                    }
                }
            }
        }
        Some(output)
    }

    fn read_parameter(&mut self, already_read_semicolon: bool) -> Option<(String, String)> {
        if !already_read_semicolon && self.next() != Some(b';') {
            return None;
        }
        self.consume_optional_lws();
        let key = match self.read_token() {
            Some(t) => t,
            None => return None,
        };
        self.consume_optional_lws();
        if self.next() != Some(b'=') {
            return None;
        }
        self.consume_optional_lws();
        let value = match self.read_token_or_quoted_string() {
            Some(t) => t,
            None => return None,
        };
        self.consume_optional_lws();
        Some((key, value))
    }

    /// Read parameters from the current position.
    ///
    /// The return value ``None`` is reserved for syntax errors in parameters that exist; a mere
    /// absense of parameters will lead to returning an empty vector instead.
    fn read_parameters(&mut self) -> Option<Vec<(String, String)>> {
        let mut result = Vec::new();
        loop {
            match self.next() {
                //This catches the LWS after the last ';', and can probably be replaced with
                //consume_optional_lws().
                Some(b) if b == SP || b == HT => (),
                Some(b) if b == b';' => {
                    match self.read_parameter(true) {
                        Some(parameter) => result.push(parameter),
                        None => return None,
                    }
                },
                Some(b) => {
                    // TODO: manually prove this; can LWS trip it up?
                    assert_eq!(self.next_byte, None);
                    self.next_byte = Some(b);
                    return Some(result);
                },
                None => return Some(result),
            }
        }
    }

    /// Read a token (RFC 2616 definition) from the header value.
    ///
    /// If no token begins at the current point of the header, ``None`` will also be returned.
    pub fn read_token_or_quoted_string(&mut self) -> Option<String> {

        let mut output = String::new();
        match self.next() {
            Some(b'"') => {
                // It is a quoted-string.
                enum State { Normal, Escaping }
                let mut state = Normal;
                loop {
                    match self.next() {
                        None => return None,
                        Some(b) => state = match state {
                            Normal if b == b'\\' => Escaping,
                            Normal if b == b'"' => break,
                            Normal | Escaping => { output.push(b as char); Normal },
                        }
                    }
                }
                return Some(output);
            },
            Some(b) => self.next_byte = Some(b),
            None => return None,
        }
        // OK, it wasn't a quoted-string. Must be a token.
        loop {
            match self.next() {
                None => break,
                Some(b) if is_separator(b) => {
                    assert_eq!(self.next_byte, None);
                    self.next_byte = Some(b);
                    break;
                },
                Some(b) if is_token_item(b) => {
                    output.push(b as char);
                },
                Some(b) => {
                    println!("TODO: what should be done with a token ended with a non-separator? \
(With token {}, {} was read.)", output, b as char);
                }
            }
        }
        if output.len() == 0 {
            None
        } else {
            Some(output)
        }
    }

    /// Read a token (RFC 2616 definition) from the header value.
    ///
    /// If no token begins at the current point of the header, ``None`` will also be returned.
    pub fn read_token(&mut self) -> Option<String> {
        let mut output = String::new();
        loop {
            match self.next() {
                None => break,
                Some(b) if is_separator(b) => {
                    assert_eq!(self.next_byte, None);
                    self.next_byte = Some(b);
                    break;
                },
                Some(b) if is_token_item(b) => {
                    output.push(b as char);
                },
                Some(b) => {
                    println!("TODO: what should be done with a token ended with a non-separator? \
(With token {}, {} was read.)", output, b as char);
                }
            }
        }
        if output.len() == 0 {
            None
        } else {
            Some(output)
        }
    }
}

impl<'a, R: Reader> Iterator<u8> for HeaderValueByteIterator<'a, R> {
    fn next(&mut self) -> Option<u8> {
        if self.state == Finished {
            return None;
        }
        loop {
            let b = match self.next_byte {
                Some(b) => {
                    self.next_byte = None;
                    b
                },
                None => match self.reader.read_byte() {
                    Err(_) => {
                        // Probably EOF; not a friendly reader :-(.
                        // Let's just call that the end. And sorry—your IoError
                        // is being concealed. Such is life with Iterator.
                        self.state = Finished;
                        return None
                    },
                    Ok(b) => b,
                }
            };
            match self.state {
                Normal if b == SP || b == HT => {
                    if self.at_start {
                        continue;
                    } else {
                        return Some(b);
                    }
                },
                Normal if b == CR => {
                    // RFC 2616 section 19.3, paragraph 3: "The line terminator for message-header
                    // fields is the sequence CRLF. However, we recommend that applications, when
                    // parsing such headers, recognize a single LF as a line terminator and ignore
                    // the leading CR."
                    continue;
                },
                Normal if b == LF => {
                    self.state = GotLF;
                    continue;
                },
                GotLF if b == SP || b == HT => {
                    // This isn't an end of header, this is LWS.
                    //
                    // RFC 2616, section 2.2:
                    //
                    //     LWS            = [CRLF] 1*( SP | HT )
                    //
                    // RFC 2616, section 4.2, paragraph 1:
                    //
                    //     Header fields can be extended over multiple lines by
                    //     preceding each extra line with at least one SP or HT.

                    self.state = Normal;
                    return Some(b);
                },
                GotLF => {
                    // Ooh! We got to a genuine end of line, so we're done.
                    // But first, we must makes sure not to lose that byte.
                    self.next_byte = Some(b);
                    self.state = Finished;
                    return None;
                },
                Normal => {
                    self.at_start = false;
                    return Some(b);
                },
                Finished => unreachable!(),
            }
        }
    }
}

/**
 * A datatype for headers.
 */
pub trait HeaderConvertible: PartialEq + Clone {
    /**
     * Read a header value from an iterator over the raw value.
     *
     * There will be no leading white space, but there may be trailing white space.
     * White space comes only in the form SP or HT; the `CR LF SP` form of whitespace is collapsed
     * to just the last character, and the `CR LF` which would indicate the end of the header line
     * in the stream is removed.
     *
     * For types that implement ``FromStr``, a sane-but-potentially-not-as-fast-as-possible default
     * would be:
     *
     * ```ignore
     * from_str(reader.collect_to_str())
     * ```
     *
     * (This is not provided as a default implementation as owing to present upstream limitations
     * that would require the type to implement FromStr also, which is not considered reasonable.)
     */
    fn from_stream<R: Reader>(reader: &mut HeaderValueByteIterator<R>) -> Option<Self>;

    /**
     * Write the HTTP value of the header to the stream.
     *
     * The default implementation uses the ``http_value`` method; for now, this should tend to be
     * enough, but there may be more efficient ways to do it in certain cases.
     */
    fn to_stream<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
        let s = self.http_value();
        writer.write(s.as_bytes())
    }

    /**
     * The value of the header as it would be written for an HTTP header.
     *
     * For types which implement ``Str``, a body of ``String::from_str(self)`` will often be sufficient.
     */
    fn http_value(&self) -> String;
}

/// A header with multiple comma-separated values. Implement this and a HeaderConvertible
/// implementation for Vec<T> is yours for free—just make sure your reading does not consume the
/// comma.
pub trait CommaListHeaderConvertible: HeaderConvertible {}

impl<T: CommaListHeaderConvertible> HeaderConvertible for Vec<T> {
    fn from_stream<R: Reader>(reader: &mut HeaderValueByteIterator<R>) -> Option<Vec<T>> {
        let mut result = Vec::new();
        loop {
            match HeaderConvertible::from_stream(reader) {
                Some(h) => result.push(h),
                None => return None,
            };
            match reader.consume_comma_lws() {
                CommaConsumed => continue,
                EndOfValue => break,
                ErrCommaNotFound => return None,
            }
        }
        Some(result)
    }

    fn to_stream<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
        for (i, item) in self.iter().enumerate() {
            if i != 0 {
                try!(writer.write(b", "))
            }
            try!(item.to_stream(writer))
        }
        Ok(())
    }

    fn http_value(&self) -> String {
        let mut out = String::new();
        for (i, item) in self.iter().enumerate() {
            if i != 0 {
                out.push_str(", ");
            }
            out.push_str(item.http_value()[])
        }
        out
    }
}

// Now let's have some common implementation types.
// Some header types really are arbitrary strings. Let's cover that case here.
impl HeaderConvertible for String {
    fn from_stream<R: Reader>(reader: &mut HeaderValueByteIterator<R>) -> Option<String> {
        Some(reader.collect_to_string())
    }

    fn to_stream<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
        writer.write(self.as_bytes())
    }

    fn http_value(&self) -> String {
        self.clone()
    }
}

impl HeaderConvertible for uint {
    fn from_stream<R: Reader>(reader: &mut HeaderValueByteIterator<R>) -> Option<uint> {
        from_str(reader.collect_to_string()[])
    }

    fn http_value(&self) -> String {
        format!("{}", self)
    }
}

impl HeaderConvertible for Url {
    fn from_stream<R: Reader>(reader: &mut HeaderValueByteIterator<R>) -> Option<Url> {
        Url::parse(reader.collect_to_string()[]).ok()
    }

    fn http_value(&self) -> String {
        format!("{}", self)
    }
}

impl CommaListHeaderConvertible for Method {}

impl HeaderConvertible for Method {
    fn from_stream<R: Reader>(reader: &mut HeaderValueByteIterator<R>) -> Option<Method> {
        match reader.read_token() {
            Some(s) => Method::from_str_or_new(s[]),
            None => None,
        }
    }

    fn http_value(&self) -> String {
        format!("{}", self)
    }
}

/*
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
    fn from_stream<R: Reader>(reader: &mut HeaderValueByteIterator<R>) -> Option<Tm> {
        let value = reader.collect_to_string();

        // XXX: %Z actually ignores any timezone other than UTC. Probably not a good idea?
        match strptime(value[], "%a, %d %b %Y %T %Z") {  // RFC 822, updated by RFC 1123
            Ok(time) => return Some(time),
            Err(_) => ()
        }

        match strptime(value[], "%A, %d-%b-%y %T %Z") {  // RFC 850, obsoleted by RFC 1036
            Ok(time) => return Some(time),
            Err(_) => ()
        }

        match strptime(value[], "%c") {  // ANSI C's asctime() format
            Ok(time) => Some(time),
            Err(_) => None
        }
    }

    fn http_value(&self) -> String {
        self.to_utc().strftime("%a, %d %b %Y %T GMT").unwrap().to_string()
    }
}

#[cfg(test)]
mod test {
    use time::Tm;
    use headers::test_utils::{from_stream_with_str, to_stream_into_str};
    use super::HeaderConvertible;

    #[test]
    fn test_from_stream_str() {
        assert_eq!(from_stream_with_str(""), Some(String::new()));
        assert_eq!(from_stream_with_str("foo \"bar baz\", yay"),
                                  Some(String::from_str("foo \"bar baz\", yay")));
    }

    #[test]
    fn test_http_value_str() {
        assert_eq!((String::new()).http_value(), String::new());
        assert_eq!((String::from_str("foo \"bar baz\", yay")).http_value(), String::from_str("foo \"bar baz\", yay"));
    }

    #[test]
    fn test_to_stream_str() {
        let s = String::new();
        assert_eq!(to_stream_into_str(&s), String::new());
        let s = String::from_str("foo \"bar baz\", yay");
        assert_eq!(to_stream_into_str(&s), String::from_str("foo \"bar baz\", yay"));
    }

    #[test]
    fn test_from_stream_uint() {
        assert_eq!(from_stream_with_str::<uint>("foo bar"), None);
        assert_eq!(from_stream_with_str::<uint>("-1"), None);
        assert_eq!(from_stream_with_str("0"), Some(0u));
        assert_eq!(from_stream_with_str("123456789"), Some(123456789u));
    }

    #[test]
    fn test_http_value_uint() {
        assert_eq!(0u.http_value(), String::from_str("0"));
        assert_eq!(123456789u.http_value(), String::from_str("123456789"));
    }

    #[test]
    fn test_to_stream_uint() {
        assert_eq!(to_stream_into_str(&0u), String::from_str("0"));
        assert_eq!(to_stream_into_str(&123456789u), String::from_str("123456789"));
    }

    fn sample_tm() -> Tm {
        Tm {
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
            tm_nsec: 0
        }
    }

    /// Test `from_stream` with an RFC 822 time (updated by RFC 1123)
    #[test]
    fn test_from_stream_rfc822() {
        assert_eq!(from_stream_with_str("Sun, 06 Nov 1994 08:49:37 GMT"), Some(sample_tm()));
    }

    /// Test `from_stream` with an RFC 850 time (obsoleted by RFC 1036)
    #[test]
    fn test_from_stream_rfc850() {
        assert_eq!(from_stream_with_str("Sunday, 06-Nov-94 08:49:37 GMT"), Some(sample_tm()));
    }

    /// Test `from_stream` with the ANSI C's asctime() format
    #[test]
    fn test_from_stream_asctime() {
        assert_eq!(from_stream_with_str("Sun Nov  6 08:49:37 1994"), Some(sample_tm()));
    }

    /// Test `from_stream` with the ANSI C's asctime() format on a single digit
    /// day with only one space (which is invalid).
    ///
    /// The spec requires *exactly* two spaces for such a day number as this
    /// and will not accept one, in spite of its dodgy LWS collapsing stance.
    /// ("All linear white space, including folding, has the same semantics as SP.")
    ///
    /// This is a test of dubious value, I'll admit, but it does make it
    /// defensible that it's deliberate, should someone complain. :P
    #[test]
    fn test_from_stream_asctime_remove_lws() {
        assert_eq!(from_stream_with_str::<Tm>("Sun Nov 6 08:49:37 1994"), None);
    }

    /// Test `from_stream` with the ANSI C's asctime() format on a two-digit day.
    ///
    /// This is necessary to contrast with `test_from_stream_asctime_remove_lws`,
    /// because a double-digit day doesn't have that padding space.
    #[test]
    fn test_from_stream_asctime_double_digit_date() {
        let mut tm = sample_tm();
        tm.tm_mday = 13;
        assert_eq!(from_stream_with_str("Sun Nov 13 08:49:37 1994"), Some(tm));
    }

    /// Test `http_value`, which outputs an RFC 1123 time
    #[test]
    fn test_http_value() {
        assert_eq!(sample_tm().http_value(), String::from_str("Sun, 06 Nov 1994 08:49:37 GMT"));
    }

    /// Test `to_stream`, which outputs an RFC 1123 time
    #[test]
    fn test_to_stream() {
        assert_eq!(to_stream_into_str(&sample_tm()), String::from_str("Sun, 06 Nov 1994 08:49:37 GMT"));
    }
}

macro_rules! headers_mod {
    {
        #[$attr:meta]
        // Not using this because of a "local ambiguity" bug
        //$($attrs:attr)*
        pub mod $mod_name:ident;
        num_headers: $num_headers:pat;
        $(
            $num_id:pat,
            $output_name:expr,
            $input_name:pat,
            $caps_ident:ident,
            $lower_ident:ident,
            $htype:ty;
        )*
    } => {
        pub mod $mod_name {
            //$($attrs;)*
            #[$attr]

            #[allow(unused_imports)]
            use std::io::{BufReader, IoResult};
            use std::ascii::OwnedAsciiExt;
            use time;
            use collections::tree_map::{TreeMap, Entries};
            use headers;
            use headers::{HeaderEnum, HeaderConvertible, HeaderValueByteIterator};

            pub enum Header {
                $($caps_ident($htype),)*
                ExtensionHeader(String, String),
            }

            #[deriving(Clone)]
            pub struct HeaderCollection {
                $(pub $lower_ident: Option<$htype>,)*
                pub extensions: TreeMap<String, String>,
            }

            impl HeaderCollection {
                pub fn new() -> HeaderCollection {
                    HeaderCollection {
                        $($lower_ident: None,)*
                        extensions: TreeMap::new(),
                    }
                }

                /// Consume a header, putting it into this structure.
                pub fn insert(&mut self, header: Header) {
                    match header {
                        $($caps_ident(value) => self.$lower_ident = Some(value),)*
                        ExtensionHeader(key, value) => { self.extensions.insert(key, value); },
                    }
                }

                /// Insert a raw header into the collection.
                /// This will return an error if the value is not valid UTF-8 or if the name is that
                /// of a strongly-typed header and the value is not a valid value for that header.
                pub fn insert_raw(&mut self, name: String, value: &[u8]) -> Result<(), ()> {
                    let mut reader = BufReader::new(value);
                    let mut value_iter = HeaderValueByteIterator::new(&mut reader);
                    match HeaderEnum::value_from_stream(name, &mut value_iter) {
                        Some(h) => {
                            self.insert(h);
                            Ok(())
                        },
                        None => Err(())
                    }
                }

                pub fn iter<'a>(&'a self) -> HeaderCollectionIterator<'a> {
                    HeaderCollectionIterator {
                        pos: 0,
                        coll: self,
                        ext_iter: None
                    }
                }

                /// Write all the headers to a writer. This includes an extra \r\n at the end to
                /// signal end of headers.
                pub fn write_all<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
                    for header in self.iter() {
                        try!(header.write_header(&mut *writer));
                    }
                    writer.write(b"\r\n")
                }
            }

            pub struct HeaderCollectionIterator<'a> {
                pos: uint,
                coll: &'a HeaderCollection,
                ext_iter: Option<Entries<'a, String, String>>
            }

            impl<'a> Iterator<Header> for HeaderCollectionIterator<'a> {
                fn next(&mut self) -> Option<Header> {
                    loop {
                        self.pos += 1;
                        match self.pos - 1 {
                            $($num_id => match self.coll.$lower_ident {
                                Some(ref v) => return Some($caps_ident(v.clone())),
                                None => continue,
                            },)*
                            $num_headers => {
                                self.ext_iter = Some(self.coll.extensions.iter());
                                continue
                            },
                            _ => match self.ext_iter.as_mut().unwrap().next() {
                                Some((k, v)) =>
                                    return Some(ExtensionHeader(k.clone(), v.clone())),
                                None => return None,
                            },
                        }
                    }
                }
            }

            impl HeaderEnum for Header {
                fn header_name(&self) -> String {
                    match *self {
                        $($caps_ident(_) => String::from_str($output_name),)*
                        ExtensionHeader(ref name, _) => name.clone(),
                    }
                }

                fn header_value(&self) -> String {
                    match *self {
                        $($caps_ident(ref h) => h.http_value(),)*
                        ExtensionHeader(_, ref value) => value.clone(),
                    }
                }

                fn write_header<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
                    match *self {
                        ExtensionHeader(ref name, ref value) => {
                            return write!(&mut *writer as &mut Writer,
                                          "{}: {}\r\n", *name, *value);
                        },
                        _ => (),
                    }

                    try!(write!(&mut *writer as &mut Writer, "{}: ", match *self {
                        $($caps_ident(_) => $output_name,)*
                        ExtensionHeader(..) => unreachable!(),  // Already returned
                    }));

                    // FIXME: all the `h` cases satisfy HeaderConvertible, can it be simplified?
                    try!(match *self {
                        $($caps_ident(ref h) => h.to_stream(writer),)*
                        ExtensionHeader(..) => unreachable!(),  // Already returned
                    });
                    write!(&mut *writer as &mut Writer, "\r\n")
                }

                fn value_from_stream<R: Reader>(name: String, value: &mut HeaderValueByteIterator<R>)
                        -> Option<Header> {
                    match name.clone().into_ascii_lower()[] {
                        $($input_name => match HeaderConvertible::from_stream(value) {
                            Some(v) => Some($caps_ident(v)),
                            None => None,
                        },)*
                        _ => Some(ExtensionHeader(name, value.collect_to_string())),
                    }
                }
            }
        }
    }
}

headers_mod! {
    #[doc = "Request whatnottery."]
    pub mod request;

    num_headers: 38;

    // RFC 2616, Section 4.5: General Header Fields
     0, "Cache-Control",     "cache-control",     CacheControl,     cache_control,     String;
     1, "Connection",        "connection",        Connection,       connection,        Vec<headers::connection::Connection>;
     2, "Date",              "date",              Date,             date,              time::Tm;
     3, "Pragma",            "pragma",            Pragma,           pragma,            String;
     4, "Trailer",           "trailer",           Trailer,          trailer,           String;
     5, "Transfer-Encoding", "transfer-encoding", TransferEncoding, transfer_encoding, Vec<headers::transfer_encoding::TransferCoding>;
     6, "Upgrade",           "upgrade",           Upgrade,          upgrade,           String;
     7, "Via",               "via",               Via,              via,               String;
     8, "Warning",           "warning",           Warning,          warning,           String;

    // RFC 2616, Section 5.3: Request Header Fields
     9, "Accept",              "accept",              Accept,             accept,              String;
    10, "Accept-Charset",      "accept-charset",      AcceptCharset,      accept_charset,      String;
    11, "Accept-Encoding",     "accept-encoding",     AcceptEncoding,     accept_encoding,     String;
    12, "Accept-Language",     "accept-language",     AcceptLanguage,     accept_language,     String;
    13, "Authorization",       "authorization",       Authorization,      authorization,       String;
    14, "Expect",              "expect",              Expect,             expect,              String;
    15, "From",                "from",                From,               from,                String;
    16, "Host",                "host",                Host,               host,                headers::host::Host;
    17, "If-Match",            "if-match",            IfMatch,            if_match,            String;
    18, "If-Modified-Since",   "if-modified-since",   IfModifiedSince,    if_modified_since,   time::Tm;
    19, "If-None-Match",       "if-none-match",       IfNoneMatch,        if_none_match,       String;
    20, "If-Range",            "if-range",            IfRange,            if_range,            String;
    21, "If-Unmodified-Since", "if-unmodified-since", IfUnmodifiedSince,  if_unmodified_since, time::Tm;
    22, "Max-Forwards",        "max-forwards",        MaxForwards,        max_forwards,        uint;
    23, "Proxy-Authorization", "proxy-authorization", ProxyAuthorization, proxy_authorization, String;
    24, "Range",               "range",               Range,              range,               String;
    25, "Referer",             "referer",             Referer,            referer,             String;
    26, "TE",                  "te",                  Te,                 te,                  String;
    27, "User-Agent",          "user-agent",          UserAgent,          user_agent,          String;

    // RFC 2616, Section 7.1: Entity Header Fields
    28, "Allow",            "allow",            Allow,           allow,            Vec<::method::Method>;
    29, "Content-Encoding", "content-encoding", ContentEncoding, content_encoding, String;
    30, "Content-Language", "content-language", ContentLanguage, content_language, String;
    31, "Content-Length",   "content-length",   ContentLength,   content_length,   uint;
    32, "Content-Location", "content-location", ContentLocation, content_location, String;
    33, "Content-MD5",      "content-md5",      ContentMd5,      content_md5,      String;
    34, "Content-Range",    "content-range",    ContentRange,    content_range,    String;
    35, "Content-Type",     "content-type",     ContentType,     content_type,     headers::content_type::MediaType;
    36, "Expires",          "expires",          Expires,         expires,          time::Tm;
    37, "Last-Modified",    "last-modified",    LastModified,    last_modified,    time::Tm;
}

headers_mod! {
    #[doc = "Response whatnottery."]
    pub mod response;

    num_headers: 29;

    // RFC 2616, Section 4.5: General Header Fields
     0, "Cache-Control",     "cache-control",     CacheControl,     cache_control,     String;
     1, "Connection",        "connection",        Connection,       connection,        Vec<headers::connection::Connection>;
     2, "Date",              "date",              Date,             date,              time::Tm;
     3, "Pragma",            "pragma",            Pragma,           pragma,            String;
     4, "Trailer",           "trailer",           Trailer,          trailer,           String;
     5, "Transfer-Encoding", "transfer-encoding", TransferEncoding, transfer_encoding, Vec<headers::transfer_encoding::TransferCoding>;
     6, "Upgrade",           "upgrade",           Upgrade,          upgrade,           String;
     7, "Via",               "via",               Via,              via,               String;
     8, "Warning",           "warning",           Warning,          warning,           String;

    // RFC 2616, Section 6.2: Response Header Fields
     9, "Accept-Patch",       "accept-patch",       AcceptPatch,       accept_patch,       String;
    10, "Accept-Ranges",      "accept-ranges",      AcceptRanges,      accept_ranges,      headers::accept_ranges::AcceptableRanges;
    11, "Age",                "age",                Age,               age,                String;
    12, "ETag",               "etag",               ETag,              etag,               headers::etag::EntityTag;
    13, "Location",           "location",           Location,          location,           ::url::Url;
    14, "Proxy-Authenticate", "proxy-authenticate", ProxyAuthenticate, proxy_authenticate, String;
    15, "Retry-After",        "retry-after",        RetryAfter,        retry_after,        String;
    16, "Server",             "server",             Server,            server,             String;
    17, "Vary",               "vary",               Vary,              vary,               String;
    18, "WWW-Authenticate",   "www-authenticate",   WwwAuthenticate,   www_authenticate,   String;

    // RFC 2616, Section 7.1: Entity Header Fields
    19, "Allow",            "allow",            Allow,           allow,            Vec<::method::Method>;
    20, "Content-Encoding", "content-encoding", ContentEncoding, content_encoding, String;
    21, "Content-Language", "content-language", ContentLanguage, content_language, String;
    22, "Content-Length",   "content-length",   ContentLength,   content_length,   uint;
    23, "Content-Location", "content-location", ContentLocation, content_location, String;
    24, "Content-MD5",      "content-md5",      ContentMd5,      content_md5,      String;
    25, "Content-Range",    "content-range",    ContentRange,    content_range,    String;
    26, "Content-Type",     "content-type",     ContentType,     content_type,     headers::content_type::MediaType;
    27, "Expires",          "expires",          Expires,         expires,          String; // TODO: Should be Tm
    28, "Last-Modified",    "last-modified",    LastModified,    last_modified,    time::Tm;
}
