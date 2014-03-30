use url::Url;
use method::{Method, Options};
use status;
use std::from_str::FromStr;
use std::io::{Stream, IoResult};
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::str;
use std::fmt;
use rfc2616::{CR, LF, SP};
use headers;
use buffer::BufferedStream;
use common::read_http_version;

use headers::{HeaderLineErr, EndOfFile, EndOfHeaders, MalformedHeaderSyntax, MalformedHeaderValue};

// /// Line/header can't be more than 4KB long (note that with the compacting of LWS the actual source
// /// data could be longer than 4KB)
// static MAX_LINE_LEN: uint = 0x1000;

static MAX_REQUEST_URI_LEN: uint = 1024;
pub static MAX_METHOD_LEN: uint = 64;

pub struct RequestBuffer<'a, S> {
    /// The socket connection to read from
    stream: &'a mut BufferedStream<S>,
}

impl<'a, S: Stream> RequestBuffer<'a, S> {
    pub fn new(stream: &'a mut BufferedStream<S>) -> RequestBuffer<'a, S> {
        RequestBuffer {
            stream: stream,
        }
    }

    pub fn read_request_line(&mut self) -> Result<(Method, RequestUri, (uint, uint)),
                                                  status::Status> {
        let method = match self.read_method() {
            Ok(m) => m,
            // TODO: this is a very common case, if a connection is kept open but then closed or
            // timed out. We should handle that case specially if we can improve perf—check if the
            // peer is still there and just drop the request if it is not
            Err(_) => return Err(status::BadRequest),
        };

        // Finished reading the method, including consuming a single SP.
        // Before we read the Request-URI, we should consume *SP (it's invalid,
        // but the spec recommends supporting it anyway).
        let mut next_byte;
        loop {
            match self.stream.read_byte() {
                Ok(_b@SP) => continue,
                Ok(b) => {
                    next_byte = b;
                    break;
                },
                _ => return Err(status::BadRequest),
            };
        }

        // Good, we're now into the Request-URI. Bear in mind that as well as
        // ending in SP, it can for HTTP/0.9 end in CR LF or LF.
        let mut raw_request_uri = ~"";
        loop {
            if next_byte == CR {
                // For CR, we must have an LF immediately afterwards.
                if self.stream.read_byte() != Ok(LF) {
                    return Err(status::BadRequest);
                } else {
                    // Simplify it by just dealing with the LF possibility
                    next_byte = LF;
                    break;
                }
            } else if next_byte == SP || next_byte == LF {
                break;
            }

            if raw_request_uri.len() == MAX_REQUEST_URI_LEN {
                return Err(status::RequestUriTooLong)
            }
            raw_request_uri.push_char(next_byte as char);

            next_byte = match self.stream.read_byte() {
                Ok(b) => b,
                _ => return Err(status::BadRequest),
            }
        }

        // Now parse it into a RequestUri.
        let request_uri = match from_str(raw_request_uri) {
            Some(r) => r,
            None => return Err(status::BadRequest),
        };

        // At this point, we need to consider what came immediately after the
        // Request-URI. If it was a SP, then we expect (again after allowing for
        // possible *SP, though illegal) to get an HTTP-Version. If it was CR LF
        // or LF, we consider it to be HTTP/0.9.
        if next_byte == LF {
            // Good, we got CR LF or LF; HTTP/0.9 it is.
            return Ok((method, request_uri, (0, 9)));
        }

        // By this point, next_byte can only be SP. Now we want an HTTP-Version.

        let mut read_b = 0;

        // FIXME: we still have one inconsistency here: this isn't trimming *SP.
        match read_http_version(self.stream, |b| { read_b = b; b == CR || b == LF }) {
            Ok(vv) if read_b == LF || self.stream.read_byte() == Ok(LF)
                => Ok((method, request_uri, vv)),  // LF or CR LF: valid
            _   => Err(status::BadRequest),  // invalid, or CR but no LF: not valid
        }
    }

    #[inline]
    fn read_method(&mut self) -> IoResult<Method> {
        include!("../generated/read_method.rs");
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
    /// - `MalformedHeaderValue`: header's value is invalid; normally, ignore it.
    /// - `MalformedHeaderSyntax`: bad request; you could drop it or try returning 400 Bad Request
    pub fn read_header<T: headers::HeaderEnum>(&mut self) -> Result<T, HeaderLineErr> {
        match headers::header_enum_from_stream(&mut *self.stream) {
        //match headers::HeaderEnum::from_stream(self.stream) {
            (Err(m), None) => Err(m),
            (Err(m), Some(b)) => {
                self.stream.poke_byte(b);
                Err(m)
            },
            (Ok(header), Some(b)) => {
                self.stream.poke_byte(b);
                Ok(header)
            }
            (Ok(header), None) => {
                // This should have read an extra byte, on account of the CR LF SP possibility
                error!("header with no next byte, did reading go wrong?");
                Ok(header)
            }
        }
    }
}

impl<'a, S: Stream> Reader for RequestBuffer<'a, S> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.stream.read(buf)
    }
}

#[test]
fn test_read_request_line() {
    use method::{Get, Options, Connect, ExtensionMethod};
    use buffer::BufferedStream;
    use memstream::MemReaderFakeStream;

    macro_rules! tt(
        ($value:expr => $expected:expr) => {{
            let expected = $expected;
            let mut stream = BufferedStream::new(
                MemReaderFakeStream::new($value.as_bytes().to_owned()));
            assert_eq!(RequestBuffer::new(&mut stream).read_request_line(), expected);
        }}
    )

    tt!("GET / HTTP/1.1\n" => Ok((Get, AbsolutePath(~"/"), (1, 1))));
    tt!("GET / HTTP/1.1\r\n" => Ok((Get, AbsolutePath(~"/"), (1, 1))));
    tt!("OPTIONS /foo/bar HTTP/1.1\r\n" => Ok((Options, AbsolutePath(~"/foo/bar"), (1, 1))));
    tt!("OPTIONS * HTTP/1.1\r\n" => Ok((Options, Star, (1, 1))));
    tt!("CONNECT example.com HTTP/1.1\r\n" => Ok((Connect,
                                                Authority(~"example.com"),
                                                (1, 1))));
    tt!("FOO /\r\n" => Ok((ExtensionMethod(~"FOO"), AbsolutePath(~"/"), (0, 9))));
    tt!("FOO /\n" => Ok((ExtensionMethod(~"FOO"), AbsolutePath(~"/"), (0, 9))));
    tt!("get    http://example.com/ HTTP/42.17\r\n"
            => Ok((ExtensionMethod(~"get"),
                    AbsoluteUri(from_str("http://example.com/").unwrap()),
                    (42, 17))));

    // Now for some failing cases.

    // method name is not a token
    tt!("GE,T / HTTP/1.1\r\n" => Err(status::BadRequest));

    // Request-URI is missing ("HTTP/1.1" isn't a valid Request-URI; I confirmed this by tracing the
    // rule through RFC 2396: the "/" prevents it from being a reg_name authority, and it doesn't
    // satisfy any of the other possibilities for Request-URI either)
    tt!("GET  HTTP/1.1\r\n" => Err(status::BadRequest));

    // Invalid HTTP-Version
    tt!("GET / HTTX/1.1\r\n" => Err(status::BadRequest));
}

/// An HTTP request sent to the server.
pub struct Request {
    /// The originating IP address of the request.
    remote_addr: Option<SocketAddr>,

    /// The host name and IP address that the request was sent to; this must always be specified for
    /// HTTP/1.1 requests (or the request will be rejected), but for HTTP/1.0 requests the Host
    /// header was not defined, and so this field will probably be None in such cases.
    //host: Option<Host>,  // Now in headers.host

    /// The headers sent with the request.
    headers: ~headers::request::HeaderCollection,

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
            match from_str(request_uri) {
                Some(url) => Some(AbsoluteUri(url)),
                None => None,
            }
        } else {
            // TODO: parse authority with extra::net::url
            Some(Authority(request_uri.to_owned()))
        }
    }
}

impl fmt::Show for RequestUri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Star => f.buf.write("*".as_bytes()),
            AbsoluteUri(ref url) => write!(f.buf, "{}", url),
            AbsolutePath(ref str) => f.buf.write(str.as_bytes()),
            Authority(ref str) => f.buf.write(str.as_bytes()),
        }
    }
}

impl Request {

    /// Get a response from an open socket.
    pub fn load(stream: &mut BufferedStream<TcpStream>) -> (~Request, Result<(), status::Status>) {
        let mut buffer = RequestBuffer::new(stream);

        // Start out with dummy values
        let mut request = ~Request {
            remote_addr: buffer.stream.wrapped.peer_name().ok(),
            headers: ~headers::request::HeaderCollection::new(),
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
            match buffer.read_header() {
                Err(EndOfFile) => fail!("client disconnected, nowhere to send response"),
                Err(EndOfHeaders) => break,
                Err(MalformedHeaderSyntax) => {
                    println!("BAD REQUEST: malformed header (TODO: is this right?)");
                    return (request, Err(status::BadRequest));
                },
                Err(MalformedHeaderValue) => {
                    println!("Bad header encountered. TODO: handle this better.");
                    // Now just ignore the header
                },
                Ok(header) => {
                    request.headers.insert(header);
                },
            }
        }

        // HTTP/1.0 doesn't have Host, but HTTP/1.1 requires it
        if request.version == (1, 1) && request.headers.host.is_none() {
            println!("BAD REQUEST: host is none for HTTP/1.1 request");
            return (request, Err(status::BadRequest));
        }

        request.close_connection = close_connection;
        match request.headers.connection {
            Some(ref h) => for v in h.iter() {
                match *v {
                    headers::connection::Close => {
                        request.close_connection = true;
                        break;
                    },
                    headers::connection::Token(ref s) if s.as_slice() == "keep-alive" => {
                        request.close_connection = false;
                        // No break; let it be overridden by close should some weird person do that
                    },
                    headers::connection::Token(_) => (),
                }
            },
            None => (),
        }

        // Read body if its length is specified
        match request.headers.content_length {
            Some(length) => {
                match buffer.read_exact(length) {
                    Ok(body) => match str::from_utf8(body) {
                        Some(body_str) => request.body = body_str.to_owned(),
                        None => return (request, Err(status::BadRequest))
                    },
                    Err(_) => return (request, Err(status::BadRequest))
                }
            },
            None => ()
        }

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
