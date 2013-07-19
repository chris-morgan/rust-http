use extra::treemap::TreeMap;
//use headers::Headers;
use std::rt;
use std::rt::io::Writer;

use std::rt::io::net::tcp::TcpStream;

use super::status;
use super::headers;

/**
 * The HTTP version tag which will be used for the response.
 *
 * At present, responses will always respond with `HTTP/1.1`, as there doesn't
 * seem much value in responding HTTP/1.0 when we don't really support it.
 * Others do this too, so there's my justification.
 */
static RESPONSE_HTTP_VERSION: &'static str = "HTTP/1.1";
// Maybe we could provide a response interface

fn to_owned_bytes(s: ~str) -> ~[u8] {
    // XXX: Rust compiler bug(?) prevents me from doing `s.as_bytes().to_owned()` in one step
    let b = s.as_bytes();
    b.to_owned()
}

macro_rules! bfmt (
        ($($expr:expr),*) => (to_owned_bytes(fmt!($($expr),*)))
        )

/*
/**
 * A HTTP Response.
 *
 * - `headers`: A hashmap with the headers to send (Content-type and -length are done seperatley).
 * - `status` - A status code for the response. For now only used for the first lne of the response.
 * /
pub struct Response {
    headers: ~TreeMap<~str, ~str>,
    status: status::Status,
}

impl Response {

    fn init(@mut self) {
        // Default to text/html
        //self.headers = ~TreeMap<~str, ~str>::new();
        self.set_header(~"Content-Type", ~"text/html");
    }

    fn write_status(&self, socket: &TcpSocket) {
        socket.write(to_owned_bytes(fmt!("%s %?\r\n", RESPONSE_HTTP_VERSION, self.status)));
    }

    fn write_headers(&self, socket: &TcpSocket) {
        for self.headers.iter().advance |(key, value)| {
            socket.write(to_owned_bytes(fmt!("%s: %s\r\n", *key, *value)));
        }
        socket.write(to_owned_bytes(~"\r\n"));
    }

    fn set_header<V: ToStr>(@mut self, name: ~str, value: V) {
        // TODO: apply some normalisation to the headers
        self.headers.insert(name, value.to_str());
    }

    fn auto_content_length(@mut self) {
        // Returns true if the key did not already exist in the map, but we don't care
        self.set_header(~"Content-Length", self.content.len());
    }

    /// Write the response to a socket using the HTTP version specified by version_number.
    // TODO: is this different for different versions?
    pub fn write(&self, version_number: (int, int), socket: &TcpSocket) {
        self.write_status(socket);
        self.write_headers(socket);
        socket.write(to_owned_bytes(copy self.content));
    }
}*/

pub struct ResponseWriter {
    // The place to write to (typically a TCP stream, rt::io::net::tcp::TcpStream)
    priv writer: TcpStream,
    priv headers_written: bool,
    headers: ~headers::Headers,
    status: status::Status,
}

impl ResponseWriter {
    /// Create a `ResponseWriter` writing to the specified location
    pub fn new(writer: TcpStream) -> ResponseWriter {
        ResponseWriter {
            writer: writer,
            headers_written: false,
            //headers: headers::Headers::new(),
            headers: ~TreeMap::new(),
            status: status::Ok,
        }
    }

    /// Write a response with the specified Content-Type and content; the Content-Length header is
    /// set based upon the contents
    pub fn write_content_auto(&mut self, content_type: ~str, content: ~str) {
        self.headers.insert(~"Content-Type", content_type);
        let cbytes = content.as_bytes();
        self.headers.insert(~"Content-Length", cbytes.len().to_str());
        self.write_headers();
        self.write(cbytes);
    }

    /// Write the Status-Line and headers of the response, in preparation for writing the body.
    ///
    /// If the headers have already been written, this will fail.
    pub fn write_headers(&mut self) {
        // This marks the beginning of the response (RFC2616 §6)
        if self.headers_written {
            fail!("ResponseWriter.write_headers() called, but headers already written");
        }

        // TODO: compare performance of bfmt!("foo %s", ...) and write(bytes!("foo ")) write(...)
        // (Titchy case, but could be good to get an idea of the perf characteristics; there's an
        // extremely remote possibility it might affect things later.)

        // Write the Status-Line (RFC2616 §6.1)
        // XXX: might be better not to hardcode HTTP/1.1.
        self.writer.write(bfmt!("HTTP/1.1 %s\r\n", self.status.to_str()));

        // Write the miscellaneous varieties of headers
        // XXX: this is not in the slightest bit sufficient; much more filtration is required.
        for self.headers.iter().advance |(name, value)| {
            self.writer.write(bfmt!("%s: %s\r\n", *name, *value));
        }
        self.writer.write(bytes!("\r\n"));
        self.headers_written = true;
    }
}

impl rt::io::Writer for ResponseWriter {

    pub fn write(&mut self, buf: &[u8]) {
        if (!self.headers_written) {
            self.write_headers();
        }
        self.writer.write(buf);
    }

    pub fn flush(&mut self) {
        self.writer.flush();
    }

}
