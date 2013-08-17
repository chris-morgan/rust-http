use std::util::unreachable;
use std::rt::io::{Reader, Writer};
use extra::url::Url;
use extra::time::Tm;
use extra::treemap::{TreeMap, TreeMapIterator};
use headers;
use headers::{HeaderEnum, HeaderConvertible, HeaderValueByteIterator};
use headers::serialization_utils::push_maybe_quoted_string;

pub enum Header {

    // RFC 2616, Section 4.5: General Header Fields
    CacheControl(~str),  //(headers::cache_control::response::CacheControl),    // Section 14.9
    Connection(headers::connection::Connection),                     // Section 14.10
    Date(Tm),                                                        // Section 14.18
    Pragma(~str),  //(headers::pragma::Pragma),                                 // Section 14.32
    Trailer(~str),  //(headers::trailer::Trailer),                              // Section 14.40
    TransferEncoding(~str),  //(headers::transfer_encoding::TransferEncoding),  // Section 14.41
    Upgrade(~str),  //(headers::upgrade::Upgrade),                              // Section 14.42
    Via(~str),  //(headers::via::Via),                                          // Section 14.45
    Warning(~str),  //(headers::warning::Warning),                              // Section 14.46

    // RFC 2616, Section 6.2: Response Header Fields
    AcceptPatch(~str),  //(headers::accept_patch::AcceptPatch),                    // RFC 5789, Section 3.1
    AcceptRanges(headers::accept_ranges::AcceptRanges),                 // Section 14.5
    Age(~str),  //(headers::age::Age),                                             // Section 14.6
    ETag(headers::etag::EntityTag),                                     // Section 14.19
    Location(Url),                                                      // Section 14.30
    ProxyAuthenticate(~str),  //(headers::proxy_authenticate::ProxyAuthenticate),  // Section 14.33
    RetryAfter(~str),  //(headers::retry_after::RetryAfter),                       // Section 14.37
    Server(~str),  //(headers::server::Server),                                    // Section 14.38
    Vary(~str),  //(headers::vary::Vary),                                          // Section 14.44
    WwwAuthenticate(~str),  //(headers::www_authenticate::WwwAuthenticate),        // Section 14.47

    // RFC 2616, Section 7.1: Entity Header Fields
    Allow(headers::allow::Allow),                                 // Section 14.7
    ContentEncoding(~str),  //(headers::content_encoding::ContentEncoding),  // Section 14.11
    ContentLanguage(~str),  //(headers::content_language::ContentLanguage),  // Section 14.12
    ContentLength(uint),                                          // Section 14.13
    ContentLocation(~str),  //(headers::content_location::ContentLocation),  // Section 14.14
    ContentMd5(~str),  //(headers::content_md5::ContentMd5),                 // Section 14.15
    ContentRange(~str),  //(headers::content_range::ContentRange),           // Section 14.16
    ContentType(~str),  //(headers::content_type::ContentType),              // Section 14.17
    Expires(Tm),                                                  // Section 14.21
    LastModified(Tm),                                             // Section 14.29
    ExtensionHeader(~str, ~str),
}

/// Intended to be used as ``response.headers``.
#[deriving(Clone)]
pub struct HeaderCollection {
    // General Header Fields
    cache_control: Option<~str>,
    connection: Option<headers::connection::Connection>,
    date: Option<Tm>,
    pragma: Option<~str>,
    trailer: Option<~str>,
    transfer_encoding: Option<~str>,
    upgrade: Option<~str>,
    via: Option<~str>,
    warning: Option<~str>,

    // Response Header Fields
    accept_patch: Option<~str>,
    accept_ranges: Option<headers::accept_ranges::AcceptRanges>,
    age: Option<~str>,
    etag: Option<headers::etag::EntityTag>,
    location: Option<Url>,
    proxy_authenticate: Option<~str>,
    retry_after: Option<~str>,
    server: Option<~str>,
    vary: Option<~str>,
    www_authenticate: Option<~str>,

    // Entity Header Fields
    allow: Option<headers::allow::Allow>,
    content_encoding: Option<~str>,
    content_language: Option<~str>,
    content_length: Option<uint>,
    content_location: Option<~str>,
    content_md5: Option<~str>,
    content_range: Option<~str>,
    content_type: Option<~str>,
    expires: Option<Tm>,
    last_modified: Option<Tm>,
    extensions: TreeMap<~str, ~str>,
}

impl HeaderCollection {
    pub fn new() -> HeaderCollection {
        HeaderCollection {
            // General Header Fields
            cache_control: None,
            connection: None,
            date: None,
            pragma: None,
            trailer: None,
            transfer_encoding: None,
            upgrade: None,
            via: None,
            warning: None,

            // Response Header Fields
            accept_patch: None,
            accept_ranges: None,
            age: None,
            etag: None,
            location: None,
            proxy_authenticate: None,
            retry_after: None,
            server: None,
            vary: None,
            www_authenticate: None,

            // Entity Header Fields
            allow: None,
            content_encoding: None,
            content_language: None,
            content_length: None,
            content_location: None,
            content_md5: None,
            content_range: None,
            content_type: None,
            expires: None,
            last_modified: None,
            extensions: TreeMap::new(),
        }
    }

    /// Consume a header, putting it into this structure.
    pub fn insert(&mut self, header: Header) {
        match header {
            // General Header Fields
            CacheControl(value) => self.cache_control = Some(value),
            Connection(value) => self.connection = Some(value),
            Date(value) => self.date = Some(value),
            Pragma(value) => self.pragma = Some(value),
            Trailer(value) => self.trailer = Some(value),
            TransferEncoding(value) => self.transfer_encoding = Some(value),
            Upgrade(value) => self.upgrade = Some(value),
            Via(value) => self.via = Some(value),
            Warning(value) => self.warning = Some(value),

            // Response Header Fields
            AcceptPatch(value) => self.accept_patch = Some(value),
            AcceptRanges(value) => self.accept_ranges = Some(value),
            Age(value) => self.age = Some(value),
            ETag(value) => self.etag = Some(value),
            Location(value) => self.location = Some(value),
            ProxyAuthenticate(value) => self.proxy_authenticate = Some(value),
            RetryAfter(value) => self.retry_after = Some(value),
            Server(value) => self.server = Some(value),
            Vary(value) => self.vary = Some(value),
            WwwAuthenticate(value) => self.www_authenticate = Some(value),

            // Entity Header Fields
            Allow(value) => self.allow = Some(value),
            ContentEncoding(value) => self.content_encoding = Some(value),
            ContentLanguage(value) => self.content_language = Some(value),
            ContentLength(value) => self.content_length = Some(value),
            ContentLocation(value) => self.content_location = Some(value),
            ContentMd5(value) => self.content_md5 = Some(value),
            ContentRange(value) => self.content_range = Some(value),
            ContentType(value) => self.content_type = Some(value),
            Expires(value) => self.expires = Some(value),
            LastModified(value) => self.last_modified = Some(value),
            ExtensionHeader(key, value) => { self.extensions.insert(key, value); },
        }
    }

    pub fn iter<'a>(&'a self) -> HeaderCollectionIterator<'a> {
        HeaderCollectionIterator { pos: 0, coll: self, ext_iter: None }
    }

    /// Write all the headers to a writer. This includes an extra \r\n at the end to signal end of
    /// headers.
    pub fn write_all<W: Writer>(&self, writer: &mut W) {
        for header in self.iter() {
            header.write_header(writer);
        }
        writer.write(bytes!("\r\n"));
    }
}

pub struct HeaderCollectionIterator<'self> {
    pos: uint,
    coll: &'self HeaderCollection,
    ext_iter: Option<TreeMapIterator<'self, ~str, ~str>>
}

impl<'self> Iterator<Header> for HeaderCollectionIterator<'self> {
    fn next(&mut self) -> Option<Header> {
        loop {
            self.pos += 1;

            match self.pos {
                // General Header Fields
                1 => match self.coll.cache_control {
                    Some(ref v) => return Some(CacheControl(v.clone())),
                    None => loop,
                },
                2 => match self.coll.connection {
                    Some(ref v) => return Some(Connection(v.clone())),
                    None => loop,
                },
                3 => match self.coll.date {
                    Some(ref v) => return Some(Date(v.clone())),
                    None => loop,
                },
                4 => match self.coll.pragma {
                    Some(ref v) => return Some(Pragma(v.clone())),
                    None => loop,
                },
                5 => match self.coll.trailer {
                    Some(ref v) => return Some(Trailer(v.clone())),
                    None => loop,
                },
                6 => match self.coll.transfer_encoding {
                    Some(ref v) => return Some(TransferEncoding(v.clone())),
                    None => loop,
                },
                7 => match self.coll.upgrade {
                    Some(ref v) => return Some(Upgrade(v.clone())),
                    None => loop,
                },
                8 => match self.coll.via {
                    Some(ref v) => return Some(Via(v.clone())),
                    None => loop,
                },
                9 => match self.coll.warning {
                    Some(ref v) => return Some(Warning(v.clone())),
                    None => loop,
                },

                // Response Header Fields
                10 => match self.coll.accept_patch {
                    Some(ref v) => return Some(AcceptPatch(v.clone())),
                    None => loop,
                },
                11 => match self.coll.accept_ranges {
                    Some(ref v) => return Some(AcceptRanges(v.clone())),
                    None => loop,
                },
                12 => match self.coll.age {
                    Some(ref v) => return Some(Age(v.clone())),
                    None => loop,
                },
                13 => match self.coll.etag {
                    Some(ref v) => return Some(ETag(v.clone())),
                    None => loop,
                },
                14 => match self.coll.location {
                    Some(ref v) => return Some(Location(v.clone())),
                    None => loop,
                },
                15 => match self.coll.proxy_authenticate {
                    Some(ref v) => return Some(ProxyAuthenticate(v.clone())),
                    None => loop,
                },
                16 => match self.coll.retry_after {
                    Some(ref v) => return Some(RetryAfter(v.clone())),
                    None => loop,
                },
                17 => match self.coll.server {
                    Some(ref v) => return Some(Server(v.clone())),
                    None => loop,
                },
                18 => match self.coll.vary {
                    Some(ref v) => return Some(Vary(v.clone())),
                    None => loop,
                },
                19 => match self.coll.www_authenticate {
                    Some(ref v) => return Some(WwwAuthenticate(v.clone())),
                    None => loop,
                },

                // Entity Header Fields
                20 => match self.coll.allow {
                    Some(ref v) => return Some(Allow(v.clone())),
                    None => loop,
                },
                21 => match self.coll.content_encoding {
                    Some(ref v) => return Some(ContentEncoding(v.clone())),
                    None => loop,
                },
                22 => match self.coll.content_language {
                    Some(ref v) => return Some(ContentLanguage(v.clone())),
                    None => loop,
                },
                23 => match self.coll.content_length {
                    Some(ref v) => return Some(ContentLength(v.clone())),
                    None => loop,
                },
                24 => match self.coll.content_location {
                    Some(ref v) => return Some(ContentLocation(v.clone())),
                    None => loop,
                },
                25 => match self.coll.content_md5 {
                    Some(ref v) => return Some(ContentMd5(v.clone())),
                    None => loop,
                },
                26 => match self.coll.content_range {
                    Some(ref v) => return Some(ContentRange(v.clone())),
                    None => loop,
                },
                27 => match self.coll.content_type {
                    Some(ref v) => return Some(ContentType(v.clone())),
                    None => loop,
                },
                28 => match self.coll.expires {
                    Some(ref v) => return Some(Expires(v.clone())),
                    None => loop,
                },
                29 => match self.coll.last_modified {
                    Some(ref v) => return Some(LastModified(v.clone())),
                    None => loop,
                },
                30 => {
                    self.ext_iter = Some(self.coll.extensions.iter());
                    loop
                },
                _ => match self.ext_iter.get_mut_ref().next() {
                    Some((k, v)) => return Some(ExtensionHeader(k.to_owned(), v.to_owned())),
                    None => return None,
                },
            }
        }
    }
}

impl HeaderEnum for Header {
    fn header_name(&self) -> ~str {
        match *self {
            // General headers
            CacheControl(*) =>     ~"Cache-Control",
            Connection(*) =>       ~"Connection",
            Date(*) =>             ~"Date",
            Pragma(*) =>           ~"Pragma",
            Trailer(*) =>          ~"Trailer",
            TransferEncoding(*) => ~"Transfer-Encoding",
            Upgrade(*) =>          ~"Upgrade",
            Via(*) =>              ~"Via",
            Warning(*) =>          ~"Warning",

            // Response headers
            AcceptPatch(*) =>       ~"Accept-Patch",
            AcceptRanges(*) =>      ~"Accept-Ranges",
            Age(*) =>               ~"Age",
            ETag(*) =>              ~"ETag",
            Location(*) =>          ~"Location",
            ProxyAuthenticate(*) => ~"Proxy-Authenticate",
            RetryAfter(*) =>        ~"Retry-After",
            Server(*) =>            ~"Server",
            Vary(*) =>              ~"Vary",
            WwwAuthenticate(*) =>   ~"WWW-Authenticate",

            // Entity headers
            Allow(*) =>           ~"Allow",
            ContentEncoding(*) => ~"Content-Encoding",
            ContentLanguage(*) => ~"Content-Language",
            ContentLength(*) =>   ~"Content-Length",
            ContentLocation(*) => ~"Content-Location",
            ContentMd5(*) =>      ~"Content-MD5",
            ContentRange(*) =>    ~"Content-Range",
            ContentType(*) =>     ~"Content-Type",
            Expires(*) =>         ~"Expires",
            LastModified(*) =>    ~"Last-Modified",
            ExtensionHeader(ref name, _) => name.to_owned(),
        }
    }

    fn header_value(&self) -> ~str {
        match *self {
            // General headers
            CacheControl(ref h) =>     h.http_value(),
            Connection(ref h) =>       h.http_value(),
            Date(ref h) =>             h.http_value(),
            Pragma(ref h) =>           h.http_value(),
            Trailer(ref h) =>          h.http_value(),
            TransferEncoding(ref h) => h.http_value(),
            Upgrade(ref h) =>          h.http_value(),
            Via(ref h) =>              h.http_value(),
            Warning(ref h) =>          h.http_value(),

            // Response headers
            AcceptPatch(ref h) =>       h.http_value(),
            AcceptRanges(ref h) =>      h.http_value(),
            Age(ref h) =>               h.http_value(),
            ETag(ref h) =>              h.http_value(),
            Location(ref h) =>          h.http_value(),
            ProxyAuthenticate(ref h) => h.http_value(),
            RetryAfter(ref h) =>        h.http_value(),
            Server(ref h) =>            h.http_value(),
            Vary(ref h) =>              h.http_value(),
            WwwAuthenticate(ref h) =>   h.http_value(),

            // Entity headers
            Allow(ref h) =>           h.http_value(),
            ContentEncoding(ref h) => h.http_value(),
            ContentLanguage(ref h) => h.http_value(),
            ContentLength(ref h) =>   h.http_value(),
            ContentLocation(ref h) => h.http_value(),
            ContentMd5(ref h) =>      h.http_value(),
            ContentRange(ref h) =>    h.http_value(),
            ContentType(ref h) =>     h.http_value(),
            Expires(ref h) =>         h.http_value(),
            LastModified(ref h) =>    h.http_value(),
            ExtensionHeader(_, ref value) => value.to_owned(),
        }
    }

    fn write_header<T: Writer>(&self, writer: &mut T) {
        match *self {
            ExtensionHeader(ref name, ref value) => {
                // TODO: be more efficient
                let mut s = ~"";
                // Allocate for name, ": " and quoted value (typically an overallocation of 2 bytes,
                // occasionally an underallocation in case of needing to escape double quotes)
                s.reserve(name.len() + 4 + value.len());
                s.push_str(*name);
                s.push_str(": ");
                let s = push_maybe_quoted_string(s, *value);
                writer.write(s.as_bytes());
                return
            },
            _ => (),
        }

        writer.write(match *self {
            // General headers
            CacheControl(*) =>     bytes!("Cache-Control: "),
            Connection(*) =>       bytes!("Connection: "),
            Date(*) =>             bytes!("Date: "),
            Pragma(*) =>           bytes!("Pragma: "),
            Trailer(*) =>          bytes!("Trailer: "),
            TransferEncoding(*) => bytes!("Transfer-Encoding: "),
            Upgrade(*) =>          bytes!("Upgrade: "),
            Via(*) =>              bytes!("Via: "),
            Warning(*) =>          bytes!("Warning: "),

            // Response headers
            AcceptPatch(*) =>       bytes!("Accept-Patch: "),
            AcceptRanges(*) =>      bytes!("Accept-Ranges: "),
            Age(*) =>               bytes!("Age: "),
            ETag(*) =>              bytes!("ETag: "),
            Location(*) =>          bytes!("Location: "),
            ProxyAuthenticate(*) => bytes!("Proxy-Authenticate: "),
            RetryAfter(*) =>        bytes!("Retry-After: "),
            Server(*) =>            bytes!("Server: "),
            Vary(*) =>              bytes!("Vary: "),
            WwwAuthenticate(*) =>   bytes!("WWW-Authenticate: "),

            // Entity headers
            Allow(*) =>           bytes!("Allow: "),
            ContentEncoding(*) => bytes!("Content-Encoding: "),
            ContentLanguage(*) => bytes!("Content-Language: "),
            ContentLength(*) =>   bytes!("Content-Length: "),
            ContentLocation(*) => bytes!("Content-Location: "),
            ContentMd5(*) =>      bytes!("Content-MD5: "),
            ContentRange(*) =>    bytes!("Content-Range: "),
            ContentType(*) =>     bytes!("Content-Type: "),
            Expires(*) =>         bytes!("Expires: "),
            LastModified(*) =>    bytes!("Last-Modified: "),
            ExtensionHeader(*) => unreachable(),  // Already returned
        });

        // FIXME: all the `h` cases satisfy HeaderConvertible, can it be simplified?
        match *self {
            // General headers
            CacheControl(ref h) =>     h.to_stream(writer),
            Connection(ref h) =>       h.to_stream(writer),
            Date(ref h) =>             h.to_stream(writer),
            Pragma(ref h) =>           h.to_stream(writer),
            Trailer(ref h) =>          h.to_stream(writer),
            TransferEncoding(ref h) => h.to_stream(writer),
            Upgrade(ref h) =>          h.to_stream(writer),
            Via(ref h) =>              h.to_stream(writer),
            Warning(ref h) =>          h.to_stream(writer),

            // Response headers
            AcceptPatch(ref h) =>       h.to_stream(writer),
            AcceptRanges(ref h) =>      h.to_stream(writer),
            Age(ref h) =>               h.to_stream(writer),
            ETag(ref h) =>              h.to_stream(writer),
            Location(ref h) =>          h.to_stream(writer),
            ProxyAuthenticate(ref h) => h.to_stream(writer),
            RetryAfter(ref h) =>        h.to_stream(writer),
            Server(ref h) =>            h.to_stream(writer),
            Vary(ref h) =>              h.to_stream(writer),
            WwwAuthenticate(ref h) =>   h.to_stream(writer),

            // Entity headers
            Allow(ref h) =>           h.to_stream(writer),
            ContentEncoding(ref h) => h.to_stream(writer),
            ContentLanguage(ref h) => h.to_stream(writer),
            ContentLength(ref h) =>   h.to_stream(writer),
            ContentLocation(ref h) => h.to_stream(writer),
            ContentMd5(ref h) =>      h.to_stream(writer),
            ContentRange(ref h) =>    h.to_stream(writer),
            ContentType(ref h) =>     h.to_stream(writer),
            Expires(ref h) =>         h.to_stream(writer),
            LastModified(ref h) =>    h.to_stream(writer),
            ExtensionHeader(*) =>     unreachable(),  // Already returned
        };
        writer.write(bytes!("\r\n"));
    }

    fn value_from_stream<T: Reader>(name: ~str, value: &mut HeaderValueByteIterator<T>)
            -> Option<Header> {
        match name.as_slice() {
            // General headers
            "Cache-Control" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(CacheControl(v)),
                None => None,
            },
            "Connection" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Connection(v)),
                None => None,
            },
            "Date" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Date(v)),
                None => None,
            },
            "Pragma" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Pragma(v)),
                None => None,
            },
            "Trailer" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Trailer(v)),
                None => None,
            },
            "Transfer-Encoding" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(TransferEncoding(v)),
                None => None,
            },
            "Upgrade" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Upgrade(v)),
                None => None,
            },
            "Via" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Via(v)),
                None => None,
            },
            "Warning" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Warning(v)),
                None => None,
            },

            // Response headers
            "Accept-Patch" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(AcceptPatch(v)),
                None => None,
            },
            "Accept-Ranges" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(AcceptRanges(v)),
                None => None,
            },
            "Age" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Age(v)),
                None => None,
            },
            "Etag" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(ETag(v)),
                None => None,
            },
            "Location" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Location(v)),
                None => None,
            },
            "Proxy-Authenticate" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(ProxyAuthenticate(v)),
                None => None,
            },
            "Retry-After" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(RetryAfter(v)),
                None => None,
            },
            "Server" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Server(v)),
                None => None,
            },
            "Vary" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Vary(v)),
                None => None,
            },
            "Www-Authenticate" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(WwwAuthenticate(v)),
                None => None,
            },

            // Entity headers
            "Allow" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Allow(v)),
                None => None,
            },
            "Content-Encoding" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(ContentEncoding(v)),
                None => None,
            },
            "Content-Language" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(ContentLanguage(v)),
                None => None,
            },
            "Content-Length" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(ContentLength(v)),
                None => None,
            },
            "Content-Location" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(ContentLocation(v)),
                None => None,
            },
            "Content-Md5" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(ContentMd5(v)),
                None => None,
            },
            "Content-Range" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(ContentRange(v)),
                None => None,
            },
            "Content-Type" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(ContentType(v)),
                None => None,
            },
            "Expires" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(Expires(v)),
                None => None,
            },
            "Last-Modified" => match HeaderConvertible::from_stream(value) {
                Some(v) => Some(LastModified(v)),
                None => None,
            },
            _ => Some(ExtensionHeader(name, value.collect_to_str())),
        }
    }
}
