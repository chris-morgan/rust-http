/*!

Things for the construction and sending of HTTP requests.

If you want to make a request, `RequestWriter::new` is where you start, and
`RequestWriter.read_response` is where you will send the request and read the response.

```rust
use http::client::RequestWriter;
use http::method::Get;

fn main() {
    let request = RequestWriter::new(Get, from_str("http://example.com/").unwrap());
    let mut response = match request.read_response() {
        Ok(response) => response,
        Err(_request) => unreachable!(), // Uncaught condition will have failed first
    };
    // Now you have a `ResponseReader`; see http::client::response for docs on that.
}
```

If you wish to send a request body (e.g. POST requests), I'm sorry to have to tell you that there is
not *good* support for this yet. However, it can be done; here is an example:

```rust
let data: ~[u8];
let mut request: RequestWriter;

request.headers.content_length = Some(data.len());
request.write(data);
let response = match request.read_response() {
    Ok(response) => response,
    Err(_request) => unreachable!(), // Uncaught condition will have failed first
};
```

*/

use extra::url::Url;
use extra::url;
use method::Method;
use std::io::{Reader, Writer};
use std::io::net::get_host_addresses;
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::net::tcp::TcpStream;
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
    // The place to write to (typically a TCP stream, io::net::tcp::TcpStream)
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
                port: Some(from_str(*p).expect("You didn’t aught to give a bad port!")),
                // TODO: fix extra::url to use u16 rather than ~str
            },
        };

        let remote_addr = url_to_socket_addr(&url);
        info!("using ip address {} for {}", remote_addr.to_str(), url.host);

        fn url_to_socket_addr(url: &Url) -> SocketAddr {
            // Just grab the first IPv4 address
            let addrs = get_host_addresses(url.host);
            // TODO: Error handling
            let addrs = addrs.unwrap();
            let addr = addrs.move_iter().find(|&a| {
                match a {
                    Ipv4Addr(..) => true,
                    _ => false
                }
            });

            // TODO: Error handling
            let addr = addr.unwrap();

            let port = url.port.clone().unwrap_or(~"80");
            let port = from_str(port);
            // TODO: Error handling
            let port = port.unwrap();

            SocketAddr {
                ip: addr,
                port: port
            }
        }

        let mut request = RequestWriter {
            stream: None,
            headers_written: false,
            remote_addr: Some(remote_addr),
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
                Some(stream) => Some(BufferedStream::new(stream)),
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
        write!(&mut self.stream as &mut Writer,
               "{} {}{}{} HTTP/1.0\r\n",
               self.method.to_str(),
               if self.url.path.len()  > 0 { self.url.path.as_slice() } else { "/" },
               if self.url.query.len() > 0 { "?" } else { "" },
               url::query_to_str(&self.url.query));

        self.headers.write_all(&mut self.stream);
        self.headers_written = true;
    }

    /**
     * Send the request and construct a `ResponseReader` out of it.
     *
     * If the request sending fails in any way, a condition will be raised; if handled, the original
     * request will be returned as an `Err`.
     */
    pub fn read_response(mut self) -> Result<ResponseReader<TcpStream>, RequestWriter<TcpStream>> {
        self.try_write_headers();
        self.flush();
        match self.stream.take() {
            Some(stream) => ResponseReader::construct(stream, self),
            None => Err(self), // TODO: raise condition
        }
    }
}

/// Write the request body. Note that any calls to `write()` will cause the headers to be sent.
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
