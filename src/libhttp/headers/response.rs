use std::util::unreachable;
use extra::url::Url;
use extra::time::Tm;
use headers;
use headers::{HeaderEnum, HeaderConvertible, HeaderValueByteIterator};
use headers::serialization_utils::{push_maybe_quoted_string, maybe_unquote_string};

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
    ETag(~str),  //(headers::etag::ETag),                                          // Section 14.19
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

impl HeaderEnum for Header {
    fn header_name<'a>(&'a self) -> &'a str {
        match *self {
            // General headers
            CacheControl(*) =>     "Cache-Control",
            Connection(*) =>       "Connection",
            Date(*) =>             "Date",
            Pragma(*) =>           "Pragma",
            Trailer(*) =>          "Trailer",
            TransferEncoding(*) => "Transfer-Encoding",
            Upgrade(*) =>          "Upgrade",
            Via(*) =>              "Via",
            Warning(*) =>          "Warning",

            // Response headers
            AcceptRanges(*) =>      "Accept-Ranges",
            Age(*) =>               "Age",
            ETag(*) =>              "ETag",
            Location(*) =>          "Location",
            ProxyAuthenticate(*) => "Proxy-Authenticate",
            RetryAfter(*) =>        "Retry-After",
            Server(*) =>            "Server",
            Vary(*) =>              "Vary",
            WwwAuthenticate(*) =>   "WWW-Authenticate",

            // Entity headers
            Allow(*) =>           "Allow",
            ContentEncoding(*) => "Content-Encoding",
            ContentLanguage(*) => "Content-Language",
            ContentLength(*) =>   "Content-Length",
            ContentLocation(*) => "Content-Location",
            ContentMd5(*) =>      "Content-MD5",
            ContentRange(*) =>    "Content-Range",
            ContentType(*) =>     "Content-Type",
            Expires(*) =>         "Expires",
            LastModified(*) =>    "Last-Modified",
            ExtensionHeader(ref name, _) => name,
        }
    }

    fn write_header<T: Writer>(&self, writer: &mut T) {
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
            ExtensionHeader(ref name, ref value) => {
                // TODO: be more efficient
                let mut s = ~"";
                // Allocate for name, ": " and quoted value (typically an overallocation of 2 bytes,
                // occasionally an underallocation in case of needing to escape double quotes)
                s.reserve(name.len() + 4 + value.len());
                s.push_str(name);
                s.push_str(": ");
                let s = push_maybe_quoted_string(s);
                writer.write(s.as_bytes());
                return
            },
        });

        // FIXME: all the `h` cases satisfy HeaderConvertible, can it be simplified?
        match *self {
            // General headers
            CacheControl(h) =>     h.to_stream(writer),
            Connection(h) =>       h.to_stream(writer),
            Date(h) =>             h.to_stream(writer),
            Pragma(h) =>           h.to_stream(writer),
            Trailer(h) =>          h.to_stream(writer),
            TransferEncoding(h) => h.to_stream(writer),
            Upgrade(h) =>          h.to_stream(writer),
            Via(h) =>              h.to_stream(writer),
            Warning(h) =>          h.to_stream(writer),

            // Response headers
            AcceptRanges(h) =>      h.to_stream(writer),
            Age(h) =>               h.to_stream(writer),
            ETag(h) =>              h.to_stream(writer),
            Location(h) =>          h.to_stream(writer),
            ProxyAuthenticate(h) => h.to_stream(writer),
            RetryAfter(h) =>        h.to_stream(writer),
            Server(h) =>            h.to_stream(writer),
            Vary(h) =>              h.to_stream(writer),
            WwwAuthenticate(h) =>   h.to_stream(writer),

            // Entity headers
            Allow(h) =>           h.to_stream(writer),
            ContentEncoding(h) => h.to_stream(writer),
            ContentLanguage(h) => h.to_stream(writer),
            ContentLength(h) =>   h.to_stream(writer),
            ContentLocation(h) => h.to_stream(writer),
            ContentMd5(h) =>      h.to_stream(writer),
            ContentRange(h) =>    h.to_stream(writer),
            ContentType(h) =>     h.to_stream(writer),
            Expires(h) =>         h.to_stream(writer),
            LastModified(h) =>    h.to_stream(writer),
            ExtensionHeader(ref name, ref value) => unreachable(),  // Already returned
        };
        writer.write(bytes!("\r\n"));
    }

    fn value_from_stream<T: Reader>(name: ~str, value: &mut HeaderValueByteIterator<T>)
            -> Option<Header> {
        match name {
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
            normalised_name => ExtensionHeader(normalised_name, maybe_unquote_string(value)),
        }
    }
}
