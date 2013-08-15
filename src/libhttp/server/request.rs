use extra::treemap::TreeMap;
use extra::url::Url;
use method::{Method, Options};
use status;
use std::rt::io::net::ip::SocketAddr;
use std::{str, u16};
use std::either::{Either, Left, Right};
use rfc2616::{CR, LF, SP, HT, COLON};
use headers::{Headers, normalise_header_name};
use headers::host::Host;
use headers;
use buffer::BufTcpStream;
use common::read_http_version;

pub enum HeaderLineErr { EndOfFile, EndOfHeaders, MalformedHeader }

/// Line/header can't be more than 4KB long (note that with the compacting of LWS the actual source
/// data could be longer than 4KB)
static MAX_LINE_LEN: uint = 0x1000;

static MAX_REQUEST_URI_LEN: uint = 1024;
pub static MAX_METHOD_LEN: uint = 64;
static MAX_HTTP_VERSION_LEN: uint = 1024;

/// Moderately arbitrary figure: read in 64KB chunks. GET requests should never be this large.
static BUF_SIZE: uint = 0x10000;  // Let's try 64KB chunks

pub struct RequestBuffer<'self> {
    /// The socket connection to read from
    stream: &'self mut BufTcpStream,

    /// A working space for 
    line_bytes: ~[u8],
}

impl<'self> RequestBuffer<'self> {
    pub fn new<'a>(stream: &'a mut BufTcpStream) -> RequestBuffer<'a> {
        RequestBuffer {
            stream: stream,
            line_bytes: ~[0u8, ..MAX_LINE_LEN],
        }
    }

    pub fn read_request_line(&mut self) -> Result<(Method, RequestUri, (uint, uint)),
                                                  status::Status> {
        let method = match self.read_method() {
            Some(m) => m,
            // TODO: this is a very common case, if a connection is kept open but then closed or
            // timed out. We should handle that case specially if we can improve perf—check if the
            // peer is still there and just drop the request if it is not
            None => return Err(status::BadRequest),
        };

        let request_uri = match self.read_request_uri() {
            Ok(m) => m,
            Err(e) => return Err(e),
        };

        match (read_http_version(self.stream, CR), self.stream.read_byte()) {
            (Some(vv), Some(b)) if b == LF => Ok((method, request_uri, vv)),
            _ => return Err(status::BadRequest),
        }
    }

    #[inline]
    fn read_method(&mut self) -> Option<Method> {
        include!("../generated/read_method.rs");
    }

    #[inline]
    fn read_request_uri(&mut self) -> Result<RequestUri, status::Status> {
        // Got that, including consuming the SP; now get the request_uri
        let mut request_uri = ~"";
        loop {
            match self.stream.read_byte() {
                None => return Err(status::BadRequest),
                Some(b) if b == SP => break,
                Some(b) => {
                    if request_uri.len() == MAX_REQUEST_URI_LEN {
                        return Err(status::RequestUriTooLong)
                    }
                    request_uri.push_char(b as char);
                }
            }
        }
        match FromStr::from_str::<RequestUri>(request_uri) {
            Some(r) => Ok(r),
            None => Err(status::BadRequest),
        }
    }

    /// Quick adapter from read_header back into read_header_line to let it compile for now
    pub fn read_header_line(&mut self) -> Result<(~str, ~str),
                                                 Either<HeaderLineErr, &'static str>> {
        match self.read_header::<headers::request::Header>() {
            Ok(headers::request::ExtensionHeader(k, v)) => Ok((k, v)),
            Ok(h) => {
                printfln!("[31;1mHeader dropped (TODO):[0m %?", h);
                Err(Right("header interpreted but I can't yet use it: dropped"))
            },
            Err(e) => Err(e),
        }
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
    pub fn read_header<T: headers::HeaderThingummy>(&mut self) -> Result<T,
                                            Either<HeaderLineErr, &'static str>> {
        enum State2 { Normal, CompactingLWS, GotCR, GotCRLF };
        // XXX: not called State because of https://github.com/mozilla/rust/issues/7770
        // TODO: investigate quoted strings

        let mut state = Normal;
        let mut in_name = true;
        let mut header_name = ~"";
        self.line_bytes.clear();

        loop {
            state = match self.stream.read_byte() {
                None => return Err(Left(EndOfFile)),
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
                        return Err(Left(EndOfHeaders));
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
                        return Err(Left(MalformedHeader));
                    },
                    GotCRLF => {
                        // Ooh! We got to a genuine end of line, so we're done
                        // But sorry, we don't want that byte after all...
                        self.stream.poke_byte(b);
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
        match headers::HeaderThingummy::from_name_and_value::<T>(header_name, str::from_bytes(self.line_bytes)) {
            Ok(h) => Ok(h),
            Err(m) => Err(Right(m)),
        }
    }
}

/// An HTTP request sent to the server.
pub struct Request {
    /// The originating IP address of the request.
    remote_addr: Option<SocketAddr>,

    /// The host name and IP address that the request was sent to; this must always be specified for
    /// HTTP/1.1 requests (or the request will be rejected), but for HTTP/1.0 requests the Host
    /// header was not defined, and so this field will probably be None in such cases.
    host: Option<Host>,

    /// The headers sent with the request.
    headers: ~Headers,

    /// The body of the request; empty for such methods as GET.
    body: ~str,

    /// The HTTP method for the request.
    method: Method,

    /// The URI that was requested, as found in the Request-URI of the Request-Line.
    /// You will almost never need to use this; you should prefer the `url` field instead.
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
    fn from_str(request_uri: &str) -> Option<RequestUri> {
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

impl Request {

    /// Get a response from an open socket.
    pub fn load(stream: &mut BufTcpStream) -> (~Request, Result<(), status::Status>) {
        let mut buffer = RequestBuffer::new(stream);

        // Start out with dummy values
        let mut request = ~Request {
            remote_addr: buffer.stream.wrapped.peer_name(),
            host: None,
            headers: ~TreeMap::new(),
            body: ~"",
            method: Options,
            request_uri: Star,
            close_connection: true,
            version: (0, 0),
        };

        let (method, request_uri, version) = match buffer.read_request_line() {
            Ok(vals) => vals,
            Err(err) => return (request, Err(err)),
        };
        request.method = method;
        request.request_uri = request_uri;
        request.version = version;

        // request.close_connection is deliberately left set to true so that in case of a bad
        // request we can close the connection
        let close_connection = match version {
            (1, 0) => true,
            (1, 1) => false,
            _ => return (request, Err(status::HttpVersionNotSupported)),
        };

        loop {
            match buffer.read_header_line() {
                Err(Left(EndOfFile)) => fail!("client disconnected, nowhere to send response"),
                Err(Left(EndOfHeaders)) => break,
                Err(Left(MalformedHeader)) => {
                    return (request, Err(status::BadRequest));
                },
                Err(Right(cause)) => {
                    printfln!("Bad header encountered (%s). TODO: handle this better.", cause);
                    // Now just ignore the header
                },
                Ok((name, value)) => {
                    let normalised = normalise_header_name(name);
                    if normalised == ~"Host" {
                        // TODO: this doesn't support IPv6 address access (e.g. "Host: [::1]")
                        // Not bothering now as this will be thrown away in favour of parse-time
                        // handling, where it's easier.
                        let mut hi = value.splitn_iter(':', 1);
                        request.host = Some(Host {
                            name: hi.next().unwrap().to_owned(),
                            port: match hi.next() {
                                Some(name) => match u16::from_str(name) {
                                    Some(port) => Some(port),
                                    None => return (request, Err(status::BadRequest)),
                                },
                                None => None,
                            },
                        });
                    }
                    request.headers.insert(normalise_header_name(name), value);
                },
            }
        }

        // HTTP/1.0 doesn't have Host, but HTTP/1.1 requires it
        if request.version == (1, 1) && request.host.is_none() {
            return (request, Err(status::BadRequest));
        }

        request.close_connection = match request.headers.find(&~"Connection") {
            Some(s) => match s.to_ascii().to_lower().to_str_ascii() {
                ~"close" => true,
                ~"keep-alive" => false,
                _ => close_connection,
            },
            None => close_connection,
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
