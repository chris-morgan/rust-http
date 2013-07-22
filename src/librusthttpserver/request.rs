use extra::treemap::TreeMap;
use extra::net::url::Url;
use super::method::{Method, Options};
use super::status;
use std::rt::io::net::tcp::TcpStream;
use std::rt::io::Reader;
use std::{str, uint};
use super::rfc2616::{CR, LF, SP, HT, COLON};
use super::headers::{Headers, normalise_header_name};

pub enum HeaderLineErr { EndOfFile, EndOfHeaders, MalformedHeader }

/// Line/header can't be more than 4KB long (note that with the compacting of LWS the actual source
/// data could be longer than 4KB)
static MAX_LINE_LEN: uint = 0x1000;

/// Moderately arbitrary figure: read in 64KB chunks. GET requests should never be this large.
static BUF_SIZE: uint = 0x10000;  // Let's try 64KB chunks

pub struct RequestBuffer<'self> {
    /// The socket connection to read from
    priv stream: &'self mut TcpStream,

    /// A working space for 
    priv line_bytes: ~[u8],

    /// Header reading takes the first byte of the next line in its search for linear white space;
    /// clearly, we mustn't lose it...
    priv peeked_byte: Option<u8>,

    /// A buffer because TcpStream.read() is SLOW.
    priv read_buf: [u8, ..BUF_SIZE],
    priv read_buf_pos: uint,
    priv read_buf_max: uint,
}

impl<'self> RequestBuffer<'self> {
    pub fn new<'a> (stream: &'a mut TcpStream) -> RequestBuffer<'a> {
        RequestBuffer {
            stream: stream,
            line_bytes: ~[0u8, ..MAX_LINE_LEN],
            peeked_byte: None,
            read_buf: [0u8, ..BUF_SIZE],
            read_buf_pos: 0u,
            read_buf_max: 0u,
        }
    }

    #[inline]
    fn read_byte(&mut self) -> Option<u8> {
        match self.peeked_byte {
            Some(byte) => {
                self.peeked_byte = None;
                Some(byte)
            },
            None => {
                if self.read_buf_pos == self.read_buf_max {
                    match self.stream.read(self.read_buf) {
                        // Not sure if Some(0) can happen. Hope not!
                        None | Some(0) => {
                            self.read_buf_pos = 0;
                            self.read_buf_max = 0;
                            None
                        },
                        Some(i) => {
                            self.read_buf_pos = 1;
                            self.read_buf_max = i;
                            Some(self.read_buf[0])
                        },
                    }
                } else {
                    self.read_buf_pos += 1;
                    Some(self.read_buf[self.read_buf_pos - 1])
                }
            },
        }
    }

    /// Read a line ending in CRLF
    pub fn read_crlf_line(&mut self) -> ~str {
        self.line_bytes.clear();

        enum State { Normal, GotCR };
        let mut state = Normal;

        loop {
            state = match self.read_byte() {
                None => fail!("EOF"),
                Some(b) => match state {
                    Normal if b == CR => {
                        GotCR
                    },
                    GotCR if b == LF => {
                        break;
                    },
                    GotCR => {
                        self.line_bytes.push(CR);
                        self.line_bytes.push(b);
                        Normal
                    },
                    Normal => {
                        self.line_bytes.push(b);
                        Normal
                    }
                }
            };
        }
        str::from_bytes(self.line_bytes)
    }

    /// Read a header (name, value) pair.
    ///
    /// This is not necessarily just a line ending with CRLF; there are much fancier rules at work.
    /// Where appropriate (TODO, it's everywhere at present) linear whitespace is collapsed into a
    /// single space.
    ///
    /// # Error values
    ///
    /// - `EndOfHeaders`: I have no more headers to give; go forth and conquer on the body!
    /// - `EndOfFile`: socket was closed unexpectedly; probable best behavour is to drop the request
    /// - `MalformedHeader`: request is bad; you could drop it or try returning 400 Bad Request
    pub fn read_header_line(&mut self) -> Result<(~str, ~str), HeaderLineErr> {
        enum State2 { Normal, CompactingLWS, GotCR, GotCRLF };
        // XXX: not called State because of https://github.com/mozilla/rust/issues/7770
        // TODO: investigate quoted strings

        let mut state = Normal;
        let mut in_name = true;
        let mut header_name = ~"";
        self.line_bytes.clear();

        loop {
            state = match self.read_byte() {
                None => return Err(EndOfFile),
                Some(b) => match state {
                    Normal | CompactingLWS if b == CR => {
                        // It's OK to go to GotCR on CompactingLWS: if it ends up CRLFSP it'll get
                        // back to CompactingLWS, and if it ends up CRLF we didn't need the trailing
                        // whitespace anyway.
                        GotCR
                    },
                    Normal | CompactingLWS if in_name && b == COLON => {
                        // As above, don't worry about trailing whitespace.
                        // Found the colon, so switch across to value.
                        in_name = false;
                        header_name = str::from_bytes(self.line_bytes);
                        self.line_bytes.clear();
                        Normal
                    },
                    GotCR if b == LF && in_name && self.line_bytes.len() == 0 => {
                        return Err(EndOfHeaders);
                    },
                    GotCR if b == LF => {
                        GotCRLF
                    },
                    GotCR => {
                        // False alarm, CR without LF
                        self.line_bytes.push(CR);
                        self.line_bytes.push(b);
                        Normal
                    },
                    GotCRLF if b == SP || b == HT => {
                        // CR LF SP is a suitable linear whitespace, so don't stop yet
                        CompactingLWS
                    },
                    GotCRLF if in_name => {
                        // Don't worry about poking b; the request is being aborted
                        return Err(MalformedHeader);
                    },
                    GotCRLF => {
                        // Ooh! We got to a genuine end of line, so we're done
                        // But sorry, we don't want that byte after all...
                        self.peeked_byte = Some(b);
                        break;
                    },
                    Normal | CompactingLWS if b == SP || b == HT => {
                        // Start or continue to compact linear whitespace
                        CompactingLWS
                    },
                    CompactingLWS => {
                        // End of LWS, compact it down to a single space, unless it's at the start
                        // (saves a trim_left() call later)
                        if self.line_bytes.len() > 0 {
                            self.line_bytes.push(SP);
                        }
                        self.line_bytes.push(b);
                        Normal
                    },
                    Normal => {
                        self.line_bytes.push(b);
                        Normal
                    }
                },
            };
        }
        return Ok((header_name, str::from_bytes(self.line_bytes)));
    }
}

/// An HTTP request sent to the server.
pub struct Request {
    /// TODO: the originating IP of the request?
    //host: ip::IpAddr,

    /// The headers sent with the request.
    headers: ~Headers,

    /// The body of the request; empty for such methods as GET.
    body: ~str,

    /// The HTTP method for the request.
    method: Method,

    /// The URI that was requested.
    request_uri: RequestUri,

    /// Whether to close the TCP connection when the request has been served.
    /// The alternative is keeping the connection open and waiting for another request.
    close_connection: bool,

    /// The HTTP version number; typically `(1, 1)` or, less commonly, `(1, 0)`.
    version: (uint, uint)
}

/// The URI (Request-URI in RFC 2616) as specified in the Status-Line of an HTTP request
#[deriving(Eq)]
pub enum RequestUri {
    /// 'The asterisk "*" means that the request does not apply to a particular resource, but to the
    /// server itself, and is only allowed when the method used does not necessarily apply to a
    /// resource. One example would be "OPTIONS * HTTP/1.1" '
    Star,

    /// 'The absoluteURI form is REQUIRED when the request is being made to a proxy. The proxy is
    /// requested to forward the request or service it from a valid cache, and return the response.
    /// Note that the proxy MAY forward the request on to another proxy or directly to the server
    /// specified by the absoluteURI. In order to avoid request loops, a proxy MUST be able to
    /// recognize all of its server names, including any aliases, local variations, and the numeric
    /// IP address. An example Request-Line would be:
    /// "GET http://www.w3.org/pub/WWW/TheProject.html HTTP/1.1"'
    AbsoluteUri(Url),

    /// 'To allow for transition to absoluteURIs in all requests in future versions of HTTP, all
    /// HTTP/1.1 servers MUST accept the absoluteURI form in requests, even though HTTP/1.1 clients
    /// will only generate them in requests to proxies.'
    ///
    /// TODO: this shouldn't be a string; it should be further parsed. `extra::net::url` has some
    /// stuff which might help, but isn't public.
    AbsolutePath(~str),

    /// 'The authority form is only used by the CONNECT method (CONNECT).'
    ///
    /// TODO: this shouldn't be a string; it should be further parsed. `extra::net::url` has some
    /// stuff which might help, but isn't public.
    Authority(~str),
}

impl FromStr for RequestUri {
    /// Interpret a RFC2616 Request-URI
    pub fn from_str(request_uri: &str) -> Option<RequestUri> {
        if request_uri == "*" {
            Some(Star)
        } else if request_uri.starts_with("/") {
            Some(AbsolutePath(request_uri.to_owned()))
        } else if request_uri.contains("/") {
            // An authority can't have a slash in it
            match FromStr::from_str::<Url>(request_uri) {
                Some(url) => Some(AbsoluteUri(url)),
                None => None,
            }
        } else {
            // TODO: parse authority with extra::net::url
            Some(Authority(request_uri.to_owned()))
        }
    }
}

/// Parse an HTTP request line into its parts.
///
/// `parse_request_line("GET /foo HTTP/1.1") == Ok((method::Get, AbsolutePath("/foo"), (1, 1)))`
fn parse_request_line(line: &str) -> Option<(Method, RequestUri, (uint, uint))> {
    let mut words = line.word_iter();
    let method = match words.next() {
        Some(s) => Method::from_str_or_new(s),
        None => return None,
    };
    let request_uri = match words.next() {
        Some(s) => match FromStr::from_str::<RequestUri>(s) {
            Some(r) => r,
            None => return None,
        },
        None => return None,
    };
    let http_version = match words.next() {
        Some(s) => parse_http_version(s),
        None => return None,
    };
    match (words.next(), http_version) {
        (None, Some(v)) => Some((method, request_uri, v)),
        _ => None,  // More words or invalid HTTP version
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
            //    |&num| uint::from_str_radix(num, 10).get()).collect();
            let mut ints = [0u, 0u];
            for v.slice_from(5).split_iter('.').enumerate().advance |(i, strnum)| {
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

impl Request {

    /// Get a response from an open socket.
    pub fn get(buffer: &mut RequestBuffer) -> (~Request, Result<(), status::Status>) {

        // Start out with dummy values
        let mut request = ~Request {
            //host: socket.get_peer_addr(),
            headers: ~TreeMap::new(),
            body: ~"",
            method: Options,
            request_uri: Star,
            close_connection: true,
            version: (0, 0),
        };

        let (method, request_uri, version) = match parse_request_line(buffer.read_crlf_line()) {
            Some(vals) => vals,
            None => return (request, Err(status::BadRequest)),
        };
        request.method = method;
        request.request_uri = request_uri;
        request.version = version;

        request.close_connection = match version {
            (1, 0) => true,
            (1, 1) => false,
            _ => return (request, Err(status::HttpVersionNotSupported)),
        };

        loop {
            match buffer.read_header_line() {
                Err(EndOfFile) => fail!("client disconnected, nowhere to send response"),
                Err(EndOfHeaders) => break,
                Err(MalformedHeader) => {
                    request.close_connection = true;
                    return (request, Err(status::BadRequest));
                },
                Ok((name, value)) => {
                    request.headers.insert(normalise_header_name(name), value);
                },
            }
        }

        request.close_connection = match request.headers.find(&~"Connection") {
            Some(s) => match s.to_ascii().to_lower().to_str_ascii() {
                ~"close" => true,
                ~"keep-alive" => false,
                _ => request.close_connection,
            },
            None => request.close_connection,
        };

        (request, Ok(()))
    }
}



/* What follows is most of Go's net/http module's definition of Request.

pub struct Request {
    // GET, POST, etc.
    method: ~Method,

    // The URL requested, constructed from the request line and (if available)
    // the Host header.
    url: ~Url,

    // The HTTP protocol version used; typically (1, 1)
    protocol: (uint, uint),

    // Request headers, all nicely and correctly parsed.
    headers: ~Headers,

    // The message body.
    body: Reader,

    // ContentLength records the length of the associated content.
    // The value -1 indicates that the length is unknown.
    // Values >= 0 indicate that the given number of bytes may
    // be read from Body.
    // For outgoing requests, a value of 0 means unknown if Body is not nil.
    content_length: i64,

    // TransferEncoding lists the transfer encodings from outermost to
    // innermost. An empty list denotes the "identity" encoding.
    // TransferEncoding can usually be ignored; chunked encoding is
    // automatically added and removed as necessary when sending and
    // receiving requests.
    transfer_encoding: ~[~str],

    // Close indicates whether to close the connection after
    // replying to this request.
    close: bool,

    // The host on which the URL is sought.
    // Per RFC 2616, this is either the value of the Host: header
    // or the host name given in the URL itself.
    // It may be of the form "host:port".
    host: ~str,

    // Form contains the parsed form data, including both the URL
    // field's query parameters and the POST or PUT form data.
    // This field is only available after ParseForm is called.
    // The HTTP client ignores Form and uses Body instead.
    form: url.Values,

    // PostForm contains the parsed form data from POST or PUT
    // body parameters.
    // This field is only available after ParseForm is called.
    // The HTTP client ignores PostForm and uses Body instead.
    post_form: url.Values,

    // MultipartForm is the parsed multipart form, including file uploads.
    // This field is only available after ParseMultipartForm is called.
    // The HTTP client ignores MultipartForm and uses Body instead.
    multipart_form: *multipart.Form,

    // Trailer maps trailer keys to values.  Like for Header, if the
    // response has multiple trailer lines with the same key, they will be
    // concatenated, delimited by commas.
    // For server requests, Trailer is only populated after Body has been
    // closed or fully consumed.
    // Trailer support is only partially complete.
    trailer: ~Headers,

    // RemoteAddr allows HTTP servers and other software to record
    // the network address that sent the request, usually for
    // logging. This field is not filled in by ReadRequest and
    // has no defined format. The HTTP server in this package
    // sets RemoteAddr to an "IP:port" address before invoking a
    // handler.
    // This field is ignored by the HTTP client.
    remote_addr: string,

    // RequestURI is the unmodified Request-URI of the
    // Request-Line (RFC 2616, Section 5.1) as sent by the client
    // to a server. Usually the URL field should be used instead.
    // It is an error to set this field in an HTTP client request.
    request_uri: string,

    // TLS allows HTTP servers and other software to record
    // information about the TLS connection on which the request
    // was received. This field is not filled in by ReadRequest.
    // The HTTP server in this package sets the field for
    // TLS-enabled connections before invoking a handler;
    // otherwise it leaves the field nil.
    // This field is ignored by the HTTP client.
    tls: *tls.ConnectionState,
}*/
