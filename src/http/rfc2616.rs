//! Values taken from RFC 2616

/// OCTET: any 8-bit sequence of data (typechecking ensures this to be true)
#[inline]
pub fn is_octet(_: u8) -> bool { true }

/// CHAR: any US-ASCII character (octets 0 - 127)
#[inline]
pub fn is_char(octet: u8) -> bool { octet < 128 }

/// UPALPHA: any US-ASCII uppercase letter "A".."Z">
#[inline]
pub fn is_upalpha(octet: u8) -> bool { octet >= 'A' as u8 && octet <= 'Z' as u8 }

/// LOALPHA: any US-ASCII lowercase letter "a".."z">
#[inline]
pub fn is_loalpha(octet: u8) -> bool { octet >= 'a' as u8 && octet <= 'z' as u8 }

/// ALPHA: UPALPHA | LOALPHA
#[inline]
pub fn is_alpha(octet: u8) -> bool { is_upalpha(octet) || is_loalpha(octet) }

/// DIGIT: any US-ASCII digit "0".."9"
#[inline]
pub fn is_digit(octet: u8) -> bool { octet >= '0' as u8 && octet <= '9' as u8 }

/// CTL: any US-ASCII control character (octets 0 - 31) and DEL (127)
#[inline]
pub fn is_ctl(octet: u8) -> bool { octet < 32 || octet == 127 }

/// CR: US-ASCII CR, carriage return (13)
pub static CR: u8 = '\r' as u8;

/// LF: US-ASCII LF, linefeed (10)
pub static LF: u8 = '\n' as u8;

/// SP: US-ASCII SP, space (32)
pub static SP: u8 = ' ' as u8;

/// HT: US-ASCII HT, horizontal-tab (9)
pub static HT: u8 = '\t' as u8;

/// US-ASCII colon (58)
pub static COLON: u8 = ':' as u8;

/// <">: US-ASCII double-quote mark (34)
pub static DOUBLE_QUOTE: u8 = '"' as u8;

/// "\": US-ASCII backslash (92)
pub static BACKSLASH: u8 = '\\' as u8;

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

/// HEX: "A" | "B" | "C" | "D" | "E" | "F" | "a" | "b" | "c" | "d" | "e" | "f" | DIGIT
#[inline]
pub fn is_hex(octet: u8) -> bool {
    (octet >= 'A' as u8 && octet <= 'F' as u8) ||
    (octet >= 'a' as u8 && octet <= 'f' as u8) ||
    is_digit(octet)
}

/// token          = 1*<any CHAR except CTLs or separators>
#[inline]
pub fn is_token_item(o: u8) -> bool {
    is_char(o) && !is_ctl(o) && !is_separator(o)
}

#[inline]
pub fn is_token(s: &str) -> bool {
    s.bytes().all(|b| is_token_item(b))
}


/// separators: "(" | ")" | "<" | ">" | "@" | "," | ";" | ":"
///           | "\" | <"> | "/" | "[" | "]" | "?" | "=" | "{"
///           | "}" | SP | HT
#[inline]
pub fn is_separator(o: u8) -> bool {
    o == '(' as u8 || o == ')' as u8 || o == '<' as u8 || o == '>' as u8 || o == '@' as u8 ||
    o == ',' as u8 || o == ';' as u8 || o == ':' as u8 || o == '\\' as u8 || o == '"' as u8 ||
    o == '/' as u8 || o == '[' as u8 || o == ']' as u8 || o == '?' as u8 || o == '=' as u8 ||
    o == '{' as u8 || o == '}' as u8 || o == SP || o == HT
}

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

// IANA is assigned as maintaining the registry for these things:
// see https://www.iana.org/assignments/http-parameters/http-parameters.xml

mod content_coding {
    use std::fmt;
    use std::from_str::FromStr;

    /// Content-coding value tokens
    pub enum ValueToken {
        // An encoding format produced by the file compression program "gzip" (GNU zip) as described
        // in RFC 1952 [25]. This format is a Lempel-Ziv coding (LZ77) with a 32 bit CRC.
        Gzip,

        // The encoding format produced by the common UNIX file compression program "compress". This
        // format is an adaptive Lempel-Ziv-Welch coding (LZW).
        // 
        // Use of program names for the identification of encoding formats is not desirable and is
        // discouraged for future encodings. Their use here is representative of historical
        // practice, not good design. For compatibility with previous implementations of HTTP,
        // applications SHOULD consider "x-gzip" and "x-compress" to be equivalent to "gzip" and
        // "compress" respectively.
        Compress,

        // The "zlib" format defined in RFC 1950 [31] in combination with the "deflate" compression
        // mechanism described in RFC 1951 [29].
        Deflate,

        // The default (identity) encoding; the use of no transformation whatsoever. This
        // content-coding is used only in the Accept- Encoding header, and SHOULD NOT be used in the
        // Content-Encoding header.
        Identity,

        // IANA has also assigned the following currently unsupported content codings:
        //
        // - "exi": W3C Efficient XML Interchange
        // - "pack200-gzip" (Network Transfer Format for Java Archives)
    }
    impl fmt::Show for ValueToken {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.buf.write(match *self {
                Gzip => "gzip".as_bytes(),
                Compress => "compress".as_bytes(),
                Deflate => "deflate".as_bytes(),
                Identity => "identity".as_bytes(),
            })
        }
    }
    impl FromStr for ValueToken {
        fn from_str(s: &str) -> Option<ValueToken> {
            use std::ascii::StrAsciiExt;
            match s.to_ascii_lower().as_slice() {
                "gzip" => Some(Gzip),
                "compress" => Some(Compress),
                "deflate" => Some(Deflate),
                "identity" => Some(Identity),
                _ => None,
            }
        }
    }
}

mod transfer_coding {
    use std::fmt;
    
    /// Transfer-coding value tokens
    // Identity is in RFC 2616 but is withdrawn in RFC 2616 errata ID 408
    // http://www.rfc-editor.org/errata_search.php?rfc=2616&eid=408
    pub enum ValueToken {
        Chunked,   // RFC 2616, §3.6.1
        Gzip,      // See above
        Compress,  // See above
        Deflate,   // See above
    }
    impl fmt::Show for ValueToken {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.buf.write(match *self {
                Chunked => "chunked".as_bytes(),
                Gzip => "gzip".as_bytes(),
                Compress => "compress".as_bytes(),
                Deflate => "deflate".as_bytes(),
            })
        }
    }
}

// A server which receives an entity-body with a transfer-coding it does
// not understand SHOULD return 501 (Unimplemented), and close the
// connection. A server MUST NOT send transfer-codings to an HTTP/1.0
// client.

// [My note: implication of this is that I need to track HTTP version number of clients.]
