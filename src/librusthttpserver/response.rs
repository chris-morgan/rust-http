use extra::treemap::TreeMap;
//use headers::Headers;
use std::rt;
use std::rt::io::Writer;

use std::rt::io::net::tcp::TcpStream;

use super::request;
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

pub struct ResponseWriter<'self> {
    // The place to write to (typically a TCP stream, rt::io::net::tcp::TcpStream)
    priv writer: TcpStream,
    priv headers_written: bool,
    request: &'self request::Request,
    headers: ~headers::Headers,
    status: status::Status,
}

impl<'self> ResponseWriter<'self> {
    /// Create a `ResponseWriter` writing to the specified location
    pub fn new(writer: TcpStream, request: &'self request::Request) -> ResponseWriter<'self> {
        ResponseWriter {
            writer: writer,
            headers_written: false,
            request: request,
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

    /// Write the Status-Line and headers of the response, if we have not already done so.
    pub fn try_write_headers(&mut self) {
        if !self.headers_written {
            self.write_headers();
        }
    }

    /// Write the Status-Line and headers of the response, in preparation for writing the body.
    ///
    /// If the headers have already been written, this will fail. See also `try_write_headers`.
    pub fn write_headers(&mut self) {
        // This marks the beginning of the response (RFC2616 §6)
        if self.headers_written {
            fail!("ResponseWriter.write_headers() called, but headers already written");
        }

        // Write the Status-Line (RFC2616 §6.1)
        // XXX: might be better not to hardcode HTTP/1.1.
        // XXX: Rust's current lack of statement-duration lifetime handling prevents this from being
        // one statement ("error: borrowed value does not live long enough")
        let s = fmt!("HTTP/1.1 %s\r\n", self.status.to_str());
        self.writer.write(s.as_bytes());

        // Write the miscellaneous varieties of headers
        // XXX: this is not in the slightest bit sufficient; much more filtration is required.
        for self.headers.iter().advance |(name, value)| {
            let s = fmt!("%s: %s\r\n", *name, *value);
            self.writer.write(s.as_bytes());
        }
        self.writer.write(bytes!("\r\n"));
        self.headers_written = true;
    }
}

impl<'self> rt::io::Writer for ResponseWriter<'self> {

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
