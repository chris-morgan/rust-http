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
        Err((_request, error)) => fail!(":-( {}", error),
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
    Err((_request, error)) => fail!(":-( {}", error),
};
```

*/

use url;
use url::Url;
use method::Method;
use std::io::{IoError, IoResult};
use std::io::net::get_host_addresses;
use std::io::net::ip::{SocketAddr, Ipv4Addr};
use buffer::BufferedStream;
use headers::request::HeaderCollection;
use headers::host::Host;
use connecter::Connecter;

use client::response::ResponseReader;

/*impl ResponseReader {
    {
        let mut buf = [0u8, ..2000];
        match stream.read(buf) {
            None => fail!("Read error :-("),  // conditions for error interception, again
            Some(bytes_read) => {
                println!(str::from_bytes(buf.slice_to(bytes_read)));
            }
        }

        match response {
            Some(response) => Ok(response),
            None => Err(self),
        }
    }
}*/

pub struct RequestWriter<S = super::NetworkStream> {
    // The place to write to (typically a network stream, which is
    // io::net::tcp::TcpStream or an SSL wrapper around that)
    stream: Option<BufferedStream<S>>,
    headers_written: bool,

    /// The originating IP address of the request.
    pub remote_addr: Option<SocketAddr>,

    /// The host name and IP address that the request was sent to; this must always be specified for
    /// HTTP/1.1 requests (or the request will be rejected), but for HTTP/1.0 requests the Host
    /// header was not defined, and so this field will probably be None in such cases.
    //host: Host,  // Now headers.host

    /// The headers sent with the request.
    pub headers: ~HeaderCollection,

    /// The HTTP method for the request.
    pub method: Method,

    /// The URL being requested.
    pub url: Url,

    /// Should we use SSL?
    use_ssl: bool,
}

/// Low-level HTTP request writing support
///
/// Moderately hacky, and due to current limitations in the TcpStream arrangement reading cannot
/// take place until writing is completed.
///
/// At present, this only supports making one request per connection.
impl<S: Reader + Writer = super::NetworkStream> RequestWriter<S> {
    /// Create a `RequestWriter` writing to the specified location
    pub fn new(method: Method, url: Url) -> IoResult<RequestWriter<S>> {
        RequestWriter::new_request(method, url, false, true)
    }

    pub fn new_request(method: Method, url: Url, use_ssl: bool, auto_detect_ssl: bool) -> IoResult<RequestWriter<S>> {
        let host = match url.port {
            None => Host {
                name: StrBuf::from_str(url.host),
                port: None,
            },
            Some(ref p) => Host {
                name: StrBuf::from_str(url.host),
                port: Some(from_str(*p).expect("You didn’t aught to give a bad port!")),
            },
        };

        let remote_addr = try!(url_to_socket_addr(&url));
        info!("using ip address {} for {}", remote_addr.to_str(), url.host);

        fn url_to_socket_addr(url: &Url) -> IoResult<SocketAddr> {
            // Just grab the first IPv4 address
            let addrs = try!(get_host_addresses(url.host));
            let addr = addrs.move_iter().find(|&a| {
                match a {
                    Ipv4Addr(..) => true,
                    _ => false
                }
            });

            // TODO: Error handling
            let addr = addr.unwrap();

            // Default to 80, using the port specified or 443 if the protocol is HTTPS.
            let port = match url.port {
                Some(ref p) => from_str(*p).expect("You didn’t aught to give a bad port!"),
                // FIXME: case insensitivity?
                None => if url.scheme.as_slice() == "https" { 443 } else { 80 },
            };

            Ok(SocketAddr {
                ip: addr,
                port: port
            })
        }

        let mut request = RequestWriter {
            stream: None,
            headers_written: false,
            remote_addr: Some(remote_addr),
            headers: ~HeaderCollection::new(),
            method: method,
            url: url,
            use_ssl: use_ssl,
        };

        if auto_detect_ssl {
            // FIXME: case insensitivity?
            request.use_ssl = request.url.scheme.as_slice() == "https";
        }

        request.headers.host = Some(host);
        Ok(request)
    }
}

impl<S: Connecter + Reader + Writer = super::NetworkStream> RequestWriter<S> {

    /// Connect to the remote host if not already connected.
    pub fn try_connect(&mut self) -> IoResult<()> {
        if self.stream.is_none() {
            self.connect()
        } else {
            Ok(())
        }
    }

    /// Connect to the remote host; fails if already connected.
    /// Returns ``true`` upon success and ``false`` upon failure (also use conditions).
    pub fn connect(&mut self) -> IoResult<()> {
        if !self.stream.is_none() {
            fail!("I don't think you meant to call connect() twice, you know.");
        }

        self.stream = match self.remote_addr {
            Some(addr) => {
                let stream = try!(Connecter::connect(addr, self.url.host, self.use_ssl));
                Some(BufferedStream::new(stream))
            },
            None => fail!("connect() called before remote_addr was set"),
        };
        Ok(())
    }

    /// Write the Request-Line and headers of the response, if we have not already done so.
    pub fn try_write_headers(&mut self) -> IoResult<()> {
        if !self.headers_written {
            self.write_headers()
        } else {
            Ok(())
        }
    }

    /// Write the Status-Line and headers of the response, in preparation for writing the body.
    ///
    /// If the headers have already been written, this will fail. See also `try_write_headers`.
    pub fn write_headers(&mut self) -> IoResult<()> {
        // This marks the beginning of the response (RFC2616 §5)
        if self.headers_written {
            fail!("RequestWriter.write_headers() called, but headers already written");
        }
        if self.stream.is_none() {
            try!(self.connect());
        }

        // Write the Request-Line (RFC2616 §5.1)
        // TODO: get to the point where we can say HTTP/1.1 with good conscience
        try!(write!(self.stream.get_mut_ref() as &mut Writer,
            "{} {}{}{} HTTP/1.0\r\n",
            self.method.to_str(),
            if self.url.path.len()  > 0 { self.url.path.as_slice() } else { "/" },
            if self.url.query.len() > 0 { "?" } else { "" },
            url::query_to_str(&self.url.query)));

        try!(self.headers.write_all(self.stream.get_mut_ref()));
        self.headers_written = true;
        Ok(())
    }

    /**
     * Send the request and construct a `ResponseReader` out of it.
     *
     * If the request sending fails in any way, a condition will be raised; if handled, the original
     * request will be returned as an `Err`.
     */
    pub fn read_response(mut self) -> Result<ResponseReader<S>, (RequestWriter<S>, IoError)> {
        match self.try_write_headers() {
            Ok(()) => (),
            Err(err) => return Err((self, err)),
        };
        match self.flush() {
            Ok(()) => (),
            Err(err) => return Err((self, err)),
        };
        match self.stream.take() {
            Some(stream) => ResponseReader::construct(stream, self),
            None => unreachable!(), // TODO: is it genuinely unreachable?
        }
    }
}

/// Write the request body. Note that any calls to `write()` will cause the headers to be sent.
impl<S: Reader + Writer + Connecter = super::NetworkStream> Writer for RequestWriter<S> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        if !self.headers_written {
            try!(self.write_headers());
        }
        // TODO: decide whether using get_mut_ref() is sound
        // (it will cause failure if None)
        self.stream.get_mut_ref().write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        // TODO: ditto
        self.stream.get_mut_ref().flush()
    }
}
