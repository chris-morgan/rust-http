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

use std::vec;
use std::ascii::Ascii;
use extra::treemap::TreeMap;
use extra::time::Tm;
use super::rfc2616::is_token;
use super::method::Method;

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
    foreach c in name.iter() {
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
/// # Examples
///
/// ~~~ .{rust}
/// assert_eq!(comma_split(" en;q=0.8, en_AU, text/html"), ["en;q=0.8", "en_AU", "text/html"])
/// ~~~
pub fn comma_split(value: &str) -> ~[~str] {
    value.split_iter(',').transform(|w| w.trim_left().to_owned()).collect()
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

pub type Headers = TreeMap<~str, ~str>;

/*impl Headers {
    fn new() -> ~Headers {
        ~Headers(*TreeMap::new::<~str, ~str>())
    }
}*/

/* In the interests simplicity of implementation, I think we'll leave the automatic normalisation
 * out for the present. Theoretically better for performance, too, unless it causes mistakes...

/// Headers
pub struct Headers {
    priv map: TreeMap<~str, ~[~str]>,
}

impl Headers {

    /// Get the named header
    pub fn get(&self, name: &str) -> Option<~str> {
        //let name = normalise_header_name(name);
        let mut concatenated = "";
        foreach hunk in map.find(name).iter() {
            concatenated += fmt!(", %s", hunk);
        }
        concatenated
    }

    /// Get the named header, split by commas
    pub fn get_vec(&self, name: &str) -> Option<~[~str]> {
        //let name = normalise_header_name(name);
        self.map.find(name)
    }

    pub fn get_vec_mut(&mut self, name: &str) -> Option<~[~str]> {
        //let name = normalise_header_name(name);
        self.map.find_mut(name)
    }

    pub fn set(&self, name: &str, value: &str) {
        //let name = normalise_header_name(name);
        // TODO: improve this; it's probably overly simplistic
        let values: ~[~str] = value.split_iter(',').skip_while(|c| c == ' ').collect();
        self.map.insert(name, values);
    }

    pub fn set(&self, name: &str, values: &[&str]) {
        //let name = normalise_header_name(name);
        // TODO: improve this; it's probably overly simplistic
        self.map.insert(name, values);
    }

    pub fn iter<'a>(&'a self) -> TreeMapIterator<'a, K, V> {
        self.map.iter().transform(|(name, values)| {
            let mut concatonated = "";
            foreach hunk in values.iter() {
                concatenated += fmt!(", %s", hunk);
            }
            concatenated
        })
    }

    pub fn iter_vec<'a>(&'a self) -> TreeMapIterator<'a, K, V> {
        self.map.iter()
    }
}
*/

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
        assert_eq!(comma_split("foo"), ~[~"foo"]);
        assert_eq!(comma_split("foo,bar"), ~[~"foo", ~"bar"]);
        assert_eq!(comma_split(" foo;q=0.8 , bar/* "), ~[~"foo;q=0.8 ", ~"bar/* "]);
    }

    #[test]
    fn test_comma_join() {
        assert_eq!(comma_join(["foo"]), ~"foo");
        assert_eq!(comma_join(["foo", "bar"]), ~"foo, bar");
        assert_eq!(comma_join([" foo;q=0.8 ", "bar/* "]), ~" foo;q=0.8 , bar/* ");
    }
}

/* A could-be-nice: have getter and setter methods for each (or most) of these, doing the
 * appropriate type conversion: (c.f. https://en.wikipedia.org/wiki/List_of_HTTP_headers)
Access-Control-Allow-Origin
Accept-Ranges
Age
Allow
Cache-Control
Connection
Content-Encoding
Content-Language
Content-Length
Content-Location
Content-MD5
Content-Disposition
Content-Range
Content-Type
Date
ETag
Expires
Last-Modified
Link
Location
P3P
Pragma
Proxy-Authenticate
Refresh
Retry-After
Server
Set-Cookie
Status
Strict-Transport-Security
Trailer
Transfer-Encoding
Vary
Via
Warning
WWW-Authenticate

// Common non-standard ones
X-Frame-Options
X-XSS-Protection
Content-Security-Policy, X-Content-Security-Policy, X-WebKit-CSP
X-Content-Type-Options
X-Powered-By
X-UA-Compatible
*/

pub fn push_quality(mut s: ~str, quality: Option<float>) -> ~str {
    // TODO: remove second and third decimal places if zero, and use a better quality type anyway
    match quality {
        Some(qvalue) => {
            s.push_char(';');
            s.push_str(fmt!("%0.3f", qvalue))
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
pub fn quoted_string(s: &str) -> ~str {
    push_quoted_string(~"", s)
}

/// Quote a string, to turn it into an RFC 2616 quoted-string
pub fn push_quoted_string(mut s: ~str, t: &str) -> ~str {
    let i = s.len();
    s.reserve_at_least(i + t.len() + 2);
    s.push_char('"');
    foreach c in t.iter() {
        if c == '\\' || c == '"' {
            s.push_char('\\');
        }
        s.push_char(c);
    }
    s.push_char('"');
    s
}

// Takes and emits the ~str instead of the &mut str for a simpler, fluid interface
pub fn push_key_value_pair(mut s: ~str, k: &str, v: &str) -> ~str {
    s.push_char(';');
    s.push_str(k);
    s.push_char('=');
    s.push_str(v);  // TODO: CRITICAL: all very well for token, but what about quoted-string?
    s
}

pub fn push_key_value_pairs(mut s: ~str, parameters: &[(~str, ~str)]) -> ~str {
    foreach &(ref k, ref v) in parameters.iter() {
        s = push_key_value_pair(s, *k, *v);
    }
    s
}

// TODO: go through http://www.iana.org/assignments/message-headers/message-headers.xhtml
// and assess what new headers need to be supported

pub mod accept {
    pub struct MediaRange {
        // It's actually not given the name "media type" in the spec (it's unnamed).
        media_type: MediaType,
        parameters: ~[(~str, ~str)],
    }
    impl ToStr for MediaRange {
        fn to_str(&self) -> ~str {
            super::push_key_value_pairs(self.media_type.to_str(), self.parameters)
        }
    }

    pub enum MediaType {
        StarSlashStar,
        TypeSlashStar(~str),
        TypeSlashSubType(~str, ~str),
    }
    impl ToStr for MediaType {
        fn to_str(&self) -> ~str {
            match *self {
                StarSlashStar => ~"*/*",
                TypeSlashStar(ref t) => t + "/*",
                TypeSlashSubType(ref t, ref s) => t + "/" + *s,
            }
        }
    }

    pub struct Item {
        media_range: MediaRange,
        quality: Option<float>,
        extensions: ~[(~str, ~str)],
    }
    impl ToStr for Item {
        fn to_str(&self) -> ~str {
            super::push_key_value_pairs(
                super::push_quality(self.media_range.to_str(), self.quality),
                self.extensions)
        }
    }
}

pub mod accept_charset {
    pub enum Charset {
        Unknown(~str),  // TODO: use charsets properly when Rust has support
        Star,
    }
    impl ToStr for Charset {
        fn to_str(&self) -> ~str {
            match *self {
                Unknown(ref s) => s.clone(),
                Star => ~"*",
            }
        }
    }

    pub struct Item {
        charset: Charset,
        quality: Option<float>,
    }
    impl ToStr for Item {
        fn to_str(&self) -> ~str {
            super::push_quality(self.charset.to_str(), self.quality)
        }
    }
}

pub mod accept_encoding {
    pub enum Coding {
        ContentCoding(super::super::rfc2616::content_coding::ValueToken),
        Unknown(~str),
        Star,
    }
    impl ToStr for Coding {
        fn to_str(&self) -> ~str {
            match *self {
                ContentCoding(vt) => vt.to_str(),
                Unknown(ref s) => s.clone(),
                Star => ~"*",
            }
        }
    }

    pub struct Item {
        coding: Coding,
        quality: Option<float>,
    }
    impl ToStr for Item {
        fn to_str(&self) -> ~str {
            super::push_quality(self.coding.to_str(), self.quality)
        }
    }
}

pub mod accept_language {
    pub enum LanguageRange {
        Unknown(~str),  // TODO. Spec for this part: ( 1*8ALPHA *( "-" 1*8ALPHA ) )
        Star,
    }
    impl ToStr for LanguageRange {
        fn to_str(&self) -> ~str {
            match *self {
                Unknown(ref s) => s.clone(),
                Star => ~"*",
            }
        }
    }

    pub struct Item {
        language: LanguageRange,
        quality: Option<float>,
    }
    impl ToStr for Item {
        fn to_str(&self) -> ~str {
            super::push_quality(self.language.to_str(), self.quality)
        }
    }
}

pub mod accept_ranges {
    pub enum RangeUnit {
        Unknown(~str),
        Bytes,
    }
    impl ToStr for RangeUnit {
        fn to_str(&self) -> ~str {
            match *self {
                Unknown(ref s) => s.clone(),
                Bytes => ~"bytes",
            }
        }
    }
    pub enum AcceptableRanges {
        RangeUnit(RangeUnit),
        None,  // Strictly, this is not a range-unit.
    }
    impl ToStr for AcceptableRanges {
        fn to_str(&self) -> ~str {
            match *self {
                RangeUnit(ref ru) => ru.to_str(),
                None => ~"none",
            }
        }
    }
}

pub type DeltaSeconds = u64;

pub mod cache_control {
    pub enum CacheDirective {
        CacheRequestDirective(request::CacheRequestDirective),
        CacheResponseDirective(response::CacheResponseDirective),
    }
    impl ToStr for CacheDirective {
        fn to_str(&self) -> ~str {
            match *self {
                CacheRequestDirective(ref d) => d.to_str(),
                CacheResponseDirective(ref d) => d.to_str(),
            }
        }
    }

    pub mod request {
        pub enum CacheRequestDirective {
            NoCache,
            NoStore,
            MaxAge(super::super::DeltaSeconds),
            MaxStale(Option<super::super::DeltaSeconds>),
            MinFresh(super::super::DeltaSeconds),
            NoTransform,
            OnlyIfCached,
            Extension(~str, Option<~str>),
        }
        impl ToStr for CacheRequestDirective {
            fn to_str(&self) -> ~str {
                match *self {
                    NoCache => ~"no-cache",
                    NoStore => ~"no-store",
                    MaxAge(ds) => fmt!("max-age=%?", ds),
                    MaxStale(ods) => match ods {
                        Some(ds) => fmt!("max-stale=%?", ds),
                        None => ~"max-stale"
                    },
                    MinFresh(ds) => fmt!("min-fresh=%?", ds),
                    NoTransform => ~"no-transform",
                    OnlyIfCached => ~"only-if-cached",
                    Extension(ref k, ref ov) => match *ov {
                        Some(ref v) => {
                            let s = ~"";
                            super::super::push_key_value_pair(s, *k, *v)
                        }
                        None => k.to_owned(),
                    }
                }
            }
        }
    }

    pub mod response {
        pub enum CacheResponseDirective {
            Public,
            Private(~[~str]),
            NoCache(~[~str]),
            NoStore,
            NoTransform,
            MustRevalidate,
            ProxyRevalidate,
            MaxAge(super::super::DeltaSeconds),
            SMaxage(super::super::DeltaSeconds),
            Extension(~str, Option<~str>),
        }
        impl ToStr for CacheResponseDirective {
            fn to_str(&self) -> ~str {
                match *self {
                    Public => ~"no-cache",
                    Private(ref fields) => match fields.len() {
                        1 => ~"private",
                        _ => {
                            let mut s = ~"private=\"";
                            s.push_str(fields.connect(","));
                            s.push_char('"');
                            s
                        }
                    },
                    NoCache(ref fields) => match fields.len() {
                        1 => ~"no-cache",
                        _ => {
                            let mut s = ~"no-cache=\"";
                            s.push_str(fields.connect(","));
                            s.push_char('"');
                            s
                        }
                    },
                    NoStore => ~"no-store",
                    NoTransform => ~"no-transform",
                    MustRevalidate => ~"must-revalidate",
                    ProxyRevalidate => ~"proxy-revalidate",
                    MaxAge(ds) => fmt!("max-age=%?", ds),
                    SMaxage(ds) => fmt!("s-maxage=%?", ds),
                    Extension(ref k, ref ov) => match *ov {
                        Some(ref v) => {
                            let s = ~"";
                            super::super::push_key_value_pair(s, *k, *v)
                        }
                        None => k.to_owned(),
                    }
                }
            }
        }
    }
}

pub mod connection {
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
}

pub mod content_encoding {
    pub enum Coding {
        ContentCoding(super::super::rfc2616::content_coding::ValueToken),
        Unknown(~str),
    }
    impl ToStr for Coding {
        fn to_str(&self) -> ~str {
            match *self {
                ContentCoding(vt) => vt.to_str(),
                Unknown(ref s) => s.clone(),
            }
        }
    }
}

pub mod content_range {
    pub enum InstanceLength {
        InstanceLength(u64),
        Unknown,  // TODO: not happy with this, as we use Unknown in other places to mean
                  // "unrecognised", but here we actually do mean "not known"
    }
    impl ToStr for InstanceLength {
        fn to_str(&self) -> ~str {
            match *self {
                InstanceLength(i) => i.to_str(),
                Unknown => ~"*",
            }
        }
    }
    pub enum ByteRangeRespSpec {
        ByteRangeRespSpec(u64, u64),
        NotSatisfiable,
    }
    impl ToStr for ByteRangeRespSpec {
        fn to_str(&self) -> ~str {
            match *self {
                ByteRangeRespSpec(first, last) => fmt!("%s-%s", first.to_str(), last.to_str()),
                NotSatisfiable => ~"*",
            }
        }
    }
    pub struct ByteContentRangeSpec {
        byte_range_resp_spec: ByteRangeRespSpec,
        instance_length: InstanceLength,
    }
    impl ToStr for ByteContentRangeSpec {
        fn to_str(&self) -> ~str {
            let mut s = ~"bytes ";
            s.push_str(self.byte_range_resp_spec.to_str());
            s.push_char('/');
            s.push_str(self.instance_length.to_str());
            s
        }
    }
}

pub mod content_type {
    pub struct MediaType {
        type_: ~str,
        subtype: ~str,
        parameters: ~[(~str, ~str)],
    }
    impl ToStr for MediaType {
        fn to_str(&self) -> ~str {
            let s = fmt!("%s/%s", self.type_, self.subtype);
            super::push_key_value_pairs(s, self.parameters)
        }
    }
}

pub mod etag {
    pub struct ETag {
        weak: bool,
        opaque_tag: ~str,
    }
    impl ToStr for ETag {
        fn to_str(&self) -> ~str {
            if self.weak {
                let s = ~"W/";
                super::push_quoted_string(s, self.opaque_tag)
            } else {
                super::quoted_string(self.opaque_tag)
            }
        }
    }
}

// RFC 2616, Section 4.5: General Header Fields
pub enum GeneralHeader {
    CacheControl(~[cache_control::CacheDirective]),  // Section 14.9
    Connection(connection::Connection),             // Section 14.10
    Date(Tm),                // Section 14.18
    Pragma(~str),            // Section 14.32, TODO
    Trailer(~str),           // Section 14.40, TODO
    TransferEncoding(~str),  // Section 14.41, TODO
    Upgrade(~str),           // Section 14.42, TODO
    Via(~str),               // Section 14.45, TODO
    Warning(~str),           // Section 14.46, TODO
}

impl GeneralHeader {
    fn header_name(&self) -> ~str {
        match *self {
            CacheControl(*) => ~"Cache-Control",
            Connection(*) => ~"Connection",
            Date(*) => ~"Date",
            Pragma(*) => ~"Pragma",
            Trailer(*) => ~"Trailer",
            TransferEncoding(*) => ~"Transfer-Encoding",
            Upgrade(*) => ~"Upgrade",
            Via(*) => ~"Via",
            Warning(*) => ~"Warning",
        }
    }
}

// RFC 2616, Section 5.3: Request Header Fields
pub enum RequestHeader {
    Accept(~[accept::Item]),                   // Section 14.1
    AcceptCharset(~[accept_charset::Item]),    // Section 14.2
    AcceptEncoding(~[accept_encoding::Item]),  // Section 14.3
    AcceptLanguage(~[accept_language::Item]),  // Section 14.4
    Authorization(~str),       // Section 14.8, TODO
    Expect(~str),              // Section 14.20, TODO
    From(~str),                // Section 14.22, TODO
    Host(~str),                // Section 14.23, TODO
    IfMatch(~str),             // Section 14.24, TODO
    IfModifiedSince(~str),     // Section 14.25, TODO
    IfNoneMatch(~str),         // Section 14.26, TODO
    IfRange(~str),             // Section 14.27, TODO
    IfUnmodifiedSince(~str),   // Section 14.28, TODO
    MaxForwards(~str),         // Section 14.31, TODO
    ProxyAuthorization(~str),  // Section 14.34, TODO
    Range(~str),               // Section 14.35, TODO
    Referer(~str),             // Section 14.36, TODO
    Te(~str),                  // Section 14.39, TODO
    UserAgent(~str),           // Section 14.43, TODO
}

impl RequestHeader {
    fn header_name(&self) -> ~str {
        match *self {
            Accept(*) => ~"Accept",
            AcceptCharset(*) => ~"Accept-Charset",
            AcceptEncoding(*) => ~"Accept-Encoding",
            AcceptLanguage(*) => ~"Accept-Language",
            Authorization(*) => ~"Authorization",
            Expect(*) => ~"Expect",
            From(*) => ~"From",
            Host(*) => ~"Host",
            IfMatch(*) => ~"If-Match",
            IfModifiedSince(*) => ~"If-Modified-Since",
            IfNoneMatch(*) => ~"If-None-Match",
            IfRange(*) => ~"If-Range",
            IfUnmodifiedSince(*) => ~"If-Unmodified-Since",
            MaxForwards(*) => ~"Max-Forwards",
            ProxyAuthorization(*) => ~"Proxy-Authorization",
            Range(*) => ~"Range",
            Referer(*) => ~"Referer",
            Te(*) => ~"TE",
            UserAgent(*) => ~"User-Agent",
        }
    }
}

// RFC 2616, Section 6.2: Response Header Fields
pub enum ResponseHeader {
    AcceptPatch(~str),        // RFC 5789, Section 3.1, TODO
    AcceptRanges(accept_ranges::AcceptableRanges),  // Section 14.5
    Age(u64),                 // Section 14.6, TODO
    ETag(etag::ETag),         // Section 14.19, TODO
    Location(~str),           // Section 14.30, TODO
    ProxyAuthenticate(~str),  // Section 14.33, TODO
    RetryAfter(~str),         // Section 14.37, TODO
    Server(~str),             // Section 14.38, TODO
    Vary(~str),               // Section 14.44, TODO
    WwwAuthenticate(~str),    // Section 14.47, TODO
}

impl ResponseHeader {
    fn header_name(&self) -> ~str {
        match *self {
            AcceptPatch(*) => ~"Accept-Patch",
            AcceptRanges(*) => ~"Accept-Ranges",
            Age(*) => ~"Age",
            ETag(*) => ~"ETag",
            Location(*) => ~"Location",
            ProxyAuthenticate(*) => ~"Proxy-Authenticate",
            RetryAfter(*) => ~"Retry-After",
            Server(*) => ~"Server",
            Vary(*) => ~"Vary",
            WwwAuthenticate(*) => ~"WWW-Authenticate",
        }
    }
}

// RFC 2616, Section 7.1: Entity Header Fields
pub enum EntityHeader {
    Allow(~[Method]),        // Section 14.7
    ContentEncoding(~[content_encoding::Coding]),  // Section 14.11
    ContentLanguage(~[~str]),  // Section 14.12, semi-TODO
    ContentLength(u64),    // Section 14.13
    ContentLocation(~str),  // Section 14.14, TODO
    ContentMd5(~str),       // Section 14.15, TODO
    ContentRange(content_range::ByteContentRangeSpec),  // Section 14.16
    ContentType(content_type::MediaType),               // Section 14.17
    Expires(~str),          // Section 14.21, TODO
    LastModified(~str),     // Section 14.29, TODO
    ExtensionHeader(~str, ~str),
}

impl EntityHeader {
    fn header_name(&self) -> ~str {
        match *self {
            Allow(*) => ~"Allow",
            ContentEncoding(*) => ~"Content-Encoding",
            ContentLanguage(*) => ~"Content-Language",
            ContentLength(*) => ~"Content-Length",
            ContentLocation(*) => ~"Content-Location",
            ContentMd5(*) => ~"Content-MD5",
            ContentRange(*) => ~"Content-Range",
            ContentType(*) => ~"Content-Type",
            Expires(*) => ~"Expires",
            LastModified(*) => ~"Last-Modified",
            ExtensionHeader(ref k, _) => k.to_owned(),
        }
    }
}

// TODO, don't like this way of splitting it up at all...

pub enum AnyRequestHeader {
    RequestGeneralHeader(GeneralHeader),
    RequestRequestHeader(RequestHeader),
    RequestEntityHeader(EntityHeader),
}

impl AnyRequestHeader {
    fn header_name(&self) -> ~str {
        match *self {
            RequestGeneralHeader(ref h) => h.header_name(),
            RequestRequestHeader(ref h) => h.header_name(),
            RequestEntityHeader(ref h) => h.header_name(),
        }
    }
}

pub enum AnyResponseHeader {
    ResponseGeneralHeader(GeneralHeader),
    ResponseResponseHeader(ResponseHeader),
    ResponseEntityHeader(EntityHeader),
}

impl AnyResponseHeader {
    fn header_name(&self) -> ~str {
        match *self {
            ResponseGeneralHeader(ref h) => h.header_name(),
            ResponseResponseHeader(ref h) => h.header_name(),
            ResponseEntityHeader(ref h) => h.header_name(),
        }
    }
}
