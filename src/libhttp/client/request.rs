use extra::url::Url;
use method::Method;
use std::rt::io::{Reader, Writer};
use std::rt::io::net::ip::SocketAddr;
use std::rt::io::net::tcp::TcpStream;
use buffer::BufferedStream;
use headers::request::HeaderCollection;
use headers::host::Host;

use client::response::ResponseReader;

/*impl ResponseReader {
    {
        let mut buf = [0u8, ..2000];
        match stream.read(buf) {
            None => fail!("Read error :-("),  // conditions for error interception, again
            Some(bytes_read) => {
                println(str::from_bytes(buf.slice_to(bytes_read)));
            }
        }

        match response {
            Some(response) => Ok(response),
            None => Err(self),
        }
    }
}*/

pub struct RequestWriter<S> {
    // The place to write to (typically a TCP stream, rt::io::net::tcp::TcpStream)
    priv stream: Option<BufferedStream<S>>,
    priv headers_written: bool,

    /// The originating IP address of the request.
    remote_addr: Option<SocketAddr>,

    /// The host name and IP address that the request was sent to; this must always be specified for
    /// HTTP/1.1 requests (or the request will be rejected), but for HTTP/1.0 requests the Host
    /// header was not defined, and so this field will probably be None in such cases.
    //host: Host,  // Now headers.host

    /// The headers sent with the request.
    headers: ~HeaderCollection,

    /// The HTTP method for the request.
    method: Method,

    /// The URL being requested.
    url: Url,
}

/// Low-level HTTP request writing support
///
/// Moderately hacky, and due to current limitations in the TcpStream arrangement reading cannot
/// take place until writing is completed.
///
/// At present, this only supports making one request per connection.
impl<S: Reader + Writer> RequestWriter<S> {
    /// Create a `RequestWriter` writing to the specified location
    pub fn new(method: Method, url: Url) -> RequestWriter<S> {
        let host = match url.port {
            None => Host {
                name: url.host.to_owned(),
                port: None,
            },
            Some(ref p) => Host {
                name: url.host.to_owned(),
                port: Some(FromStr::from_str(*p).expect("You didn’t aught to give a bad port!")),
                // TODO: fix extra::url to use u16 rather than ~str
            },
        };

        let mut request = RequestWriter {
            stream: None,
            headers_written: false,
            remote_addr: None,
            headers: ~HeaderCollection::new(),
            method: method,
            url: url,
        };
        request.headers.host = Some(host);
        request
    }
}

impl RequestWriter<TcpStream> {

    /// Connect to the remote host if not already connected.
    pub fn try_connect(&mut self) {
        if self.stream.is_none() {
            self.connect();
        }
    }

    /// Connect to the remote host; fails if already connected.
    /// Returns ``true`` upon success and ``false`` upon failure (also use conditions).
    pub fn connect(&mut self) -> bool {
        if !self.stream.is_none() {
            fail!("I don't think you meant to call connect() twice, you know.");
        }

        self.stream = match self.remote_addr {
            Some(addr) => match TcpStream::connect(addr) {
                Some(stream) => Some(BufferedStream::new(stream, false)),
                None => return false,
            },
            None => fail!("connect() called before remote_addr was set"),
        };
        true
    }

    /// Write the Request-Line and headers of the response, if we have not already done so.
    pub fn try_write_headers(&mut self) {
        if !self.headers_written {
            self.write_headers();
        }
    }

    /// Write the Status-Line and headers of the response, in preparation for writing the body.
    ///
    /// If the headers have already been written, this will fail. See also `try_write_headers`.
    pub fn write_headers(&mut self) {
        // This marks the beginning of the response (RFC2616 §5)
        if self.headers_written {
            fail!("RequestWriter.write_headers() called, but headers already written");
        }
        if self.stream.is_none() && !self.connect() {
            fail!("Uh oh, failed to connect!"); // TODO: condition
        }

        // Write the Request-Line (RFC2616 §5.1)
        // TODO: get to the point where we can say HTTP/1.1 with good conscience
        // XXX: Rust's current lack of statement-duration lifetime handling prevents this from being
        // one statement ("error: borrowed value does not live long enough")
        // TODO: don't send the entire URL; just url.{path, query}
        let s = fmt!("%s %s HTTP/1.0\r\n", self.method.to_str(), self.url.to_str());
        self.stream.write(s.as_bytes());

        self.headers.write_all(&mut self.stream);
        self.headers_written = true;
    }

    // FIXME: ~self rather than self to work around a Rust bug in by-val self at present leading to
    // a segfault on calling construct().
    pub fn read_response(~self) -> Result<ResponseReader<TcpStream>, ~RequestWriter<TcpStream>> {
        let mut mut_self = self;
        mut_self.try_write_headers();
        mut_self.flush();
        match mut_self.stream.take() {
            Some(stream) => ResponseReader::construct(stream, mut_self),
            None => Err(mut_self), // TODO: raise condition
        }
    }
}

impl Writer for RequestWriter<TcpStream> {
    fn write(&mut self, buf: &[u8]) {
        if (!self.headers_written) {
            self.write_headers();
        }
        self.stream.write(buf);
    }

    fn flush(&mut self) {
        self.stream.flush();
    }
}
