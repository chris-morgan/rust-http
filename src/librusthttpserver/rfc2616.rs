//! Values taken from RFC 2616

use extra::time::{Tm, strptime};

/*
/// OCTET: any 8-bit sequence of data (typechecking ensures this to be true)
#[inline]
pub fn is_octet(_: u8) -> bool { true }

/// CHAR: any US-ASCII character (octets 0 - 127)
#[inline]
pub fn is_char(octet: u8) -> bool { octet < 128 }

/// UPALPHA: any US-ASCII uppercase letter "A".."Z">
#[inline]
pub fn is_upalpha(octet: u8) -> bool { octet >= 65 && octet <= 90 }

/// LOALPHA: any US-ASCII lowercase letter "a".."z">
#[inline]
pub fn is_loalpha(octet: u8) -> bool { octet >= 97 && octet <= 122 }

/// ALPHA: UPALPHA | LOALPHA
#[inline]
pub fn is_alpha(octet: u8) -> bool { is_upalpha(octet) || is_loalpha(octet) }

/// DIGIT: any US-ASCII digit "0".."9"
#[inline]
pub fn is_digit(octet: u8) -> bool { octet >= 48 && octet <= 57 }

/// CTL: any US-ASCII control character (octets 0 - 31) and DEL (127)
#[inline]
pub fn is_ctl(octet: u8) -> bool { octet < 32 || octet == 127 }
*/

/// CR: US-ASCII CR, carriage return (13)
pub static CR: u8 = 13;

/// LF: US-ASCII LF, linefeed (10)
pub static LF: u8 = 10;

/// SP: US-ASCII SP, space (32)
pub static SP: u8 = 32;

/// HT: US-ASCII HT, horizontal-tab (9)
pub static HT: u8 = 9;

/// US-ASCII colon (58)
pub static COLON: u8 = 58;

/// <">: US-ASCII double-quote mark (34)
pub static DOUBLE_QUOTE: u8 = 34;

#[inline]
pub fn is_sp_or_ht(octet: u8) -> bool { octet == SP || octet == HT }

// CRLF: CR LF
//static CRLF: [u8] = [CR, LF];

// LWS: [CRLF] 1*( SP | HT )
/*#[inline]
fn is_lws(octets: &[u8]) -> bool {
    let mut has_crlf = false;
    let mut iter = octets.iter();
    if len(octets) == 0 {
        return false;
    }
    if len(octets) > 2 && octets[0] == CR && octets[1] == LF {
        iter = iter.next().next();
    }
    iter.all(|&octet| octet == SP || octet == HT)
}
*/

/*
   The TEXT rule is only used for descriptive field contents and values
   that are not intended to be interpreted by the message parser. Words
   of *TEXT MAY contain characters from character sets other than ISO-
   8859-1 [22] only when encoded according to the rules of RFC 2047
   [14].

       TEXT           = <any OCTET except CTLs,
                        but including LWS>

   A CRLF is allowed in the definition of TEXT only as part of a header
   field continuation. It is expected that the folding LWS will be
   replaced with a single SP before interpretation of the TEXT value.
*/

/*
/// HEX: "A" | "B" | "C" | "D" | "E" | "F" | "a" | "b" | "c" | "d" | "e" | "f" | DIGIT
#[inline]
pub fn is_hex(octet: u8) -> bool {
    (octet >= 65 && octet <= 70) || (octet >= 97 && octet <= 102) || is_digit(octet)
}

/// token          = 1*<any CHAR except CTLs or separators>


/// separators: "(" | ")" | "<" | ">" | "@" | "," | ";" | ":"
///           | "\" | <"> | "/" | "[" | "]" | "?" | "=" | "{"
///           | "}" | SP | HT
#[inline]
pub fn is_separator(o: u8) -> bool {
    o == 40 || o == 41 || o == 60 || o == 62 || o == 64 || o == 44 || o == 59 || o == 58 || o == 92
        || o == 34 || o == 47 || o == 91 || o == 93 || o == 63 || o == 61 || o == 123 ||
        o == 125 || o == SP || o == HT
}
*/
/*
 * Comments can be included in some HTTP header fields by surrounding
 * the comment text with parentheses. Comments are only allowed in
 * fields containing "comment" as part of their field value definition.
 * In all other fields, parentheses are considered part of the field
 * value.
 *
 *     comment        = "(" *( ctext | quoted-pair | comment ) ")"
 *     ctext          = <any TEXT excluding "(" and ")">
 *
 * A string of text is parsed as a single word if it is quoted using
 * double-quote marks.
 *
 *     quoted-string  = ( <"> *(qdtext | quoted-pair ) <"> )
 *     qdtext         = <any TEXT except <">>
 *
 * The backslash character ("\") MAY be used as a single-character
 * quoting mechanism only within quoted-string and comment constructs.
 *
 *     quoted-pair    = "\" CHAR
 */
//#[inline]
//fn is_quoted_pair(o: &[u8, ..2]) { o[0] == 92 }

/**
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
pub fn parse_http_time(value: &str) -> Option<Tm> {
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
        Ok(time) => return Some(time),
        Err(*) => return None
    }
}

pub fn format_http_time(value: Tm) -> ~str {
    value.to_utc().strftime("%a, %d %b %Y %T GMT")
}

// IANA is assigned as maintaining the registry for these things:
// see https://www.iana.org/assignments/http-parameters/http-parameters.xml

/// Content-coding value tokens
pub enum ContentCodingValueToken {
    // An encoding format produced by the file compression program "gzip" (GNU zip) as described in
    // RFC 1952 [25]. This format is a Lempel-Ziv coding (LZ77) with a 32 bit CRC.
    CCVTGzip,

    // The encoding format produced by the common UNIX file compression program "compress". This
    // format is an adaptive Lempel-Ziv-Welch coding (LZW).
    // 
    // Use of program names for the identification of encoding formats is not desirable and is
    // discouraged for future encodings. Their use here is representative of historical practice,
    // not good design. For compatibility with previous implementations of HTTP, applications SHOULD
    // consider "x-gzip" and "x-compress" to be equivalent to "gzip" and "compress" respectively.
    CCVTCompress,

    // The "zlib" format defined in RFC 1950 [31] in combination with the "deflate" compression
    // mechanism described in RFC 1951 [29].
    CCVTDeflate,

    // The default (identity) encoding; the use of no transformation whatsoever. This content-coding
    // is used only in the Accept- Encoding header, and SHOULD NOT be used in the Content-Encoding
    // header.
    CCVTIdentity,

    // IANA has also assigned the following currently unsupported content codings:
    //
    // - "exi": W3C Efficient XML Interchange
    // - "pack200-gzip" (Network Transfer Format for Java Archives)
}

/// Transfer-coding value tokens
// Identity is in RFC 2616 but is withdrawn in RFC 2616 errata ID 408
// http://www.rfc-editor.org/errata_search.php?rfc=2616&eid=408
pub enum TransferCodingValueToken {
    TCVTChunked,   // RFC 2616, §3.6.1
    TCVTGzip,      // See above
    TCVTCompress,  // See above
    TCVTDeflate,   // See above
}

// A server which receives an entity-body with a transfer-coding it does
// not understand SHOULD return 501 (Unimplemented), and close the
// connection. A server MUST NOT send transfer-codings to an HTTP/1.0
// client.

// [My note: implication of this is that I need to track HTTP version number of clients.]

#[cfg(test)]
mod test {
    use super::*;
    use extra::time::Tm;

    fn sample_tm(zone: ~str) -> Option<Tm> {
        Some(Tm {
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

    /// Test `parse_http_time` with an RFC 822 time (updated by RFC 1123)
    #[test]
    fn test_parse_http_time_rfc822() {
        assert_eq!(parse_http_time("Sun, 06 Nov 1994 08:49:37 GMT"), sample_tm(~"UTC"));
    }

    /// Test `parse_http_time` with an RFC 850 time (obsoleted by RFC 1036)
    #[test]
    fn test_parse_http_time_rfc850() {
        assert_eq!(parse_http_time("Sunday, 06-Nov-94 08:49:37 GMT"), sample_tm(~"UTC"));
    }

    /// Test `parse_http_time` with the ANSI C's asctime() format
    #[test]
    fn test_parse_http_time_asctime() {
        assert_eq!(parse_http_time("Sun Nov  6 08:49:37 1994"), sample_tm(~""));
    }
}
