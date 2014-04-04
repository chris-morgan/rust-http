use std::io::IoResult;
use std::io::net::tcp::TcpStream;

use buffer::BufferedStream;
use server::Request;
use status;
use headers::response::HeaderCollection;
use headers::content_type::MediaType;
use headers::transfer_encoding::Chunked;

/*
 * The HTTP version tag which will be used for the response.
 *
 * At present, responses will always respond with `HTTP/1.1`, as there doesn't
 * seem much value in responding HTTP/1.0 when we don't really support it.
 * Others do this too, so there's my justification.
 */
//static RESPONSE_HTTP_VERSION: &'static str = "HTTP/1.1";
// Maybe we could provide a response interface

pub struct ResponseWriter<'a> {
    // The place to write to (typically a TCP stream, io::net::tcp::TcpStream)
    writer: &'a mut BufferedStream<TcpStream>,
    headers_written: bool,
    pub request: &'a Request,
    pub headers: ~HeaderCollection,
    pub status: status::Status,
}

impl<'a> ResponseWriter<'a> {
    /// Create a `ResponseWriter` writing to the specified location
    pub fn new(writer: &'a mut BufferedStream<TcpStream>, request: &'a Request) -> ResponseWriter<'a> {
        ResponseWriter {
            writer: writer,
            headers_written: false,
            request: request,
            headers: ~HeaderCollection::new(),
            status: status::Ok,
        }
    }

    /// Write a response with the specified Content-Type and content; the Content-Length header is
    /// set based upon the contents
    pub fn write_content_auto(&mut self, content_type: MediaType, content: ~str) -> IoResult<()> {
        self.headers.content_type = Some(content_type);
        let cbytes = content.as_bytes();
        self.headers.content_length = Some(cbytes.len());
        try!(self.write_headers());
        self.write(cbytes)
    }

    /// Write the Status-Line and headers of the response, if we have not already done so.
    pub fn try_write_headers(&mut self) -> IoResult<()> {
        if !self.headers_written {
            self.write_headers()
        } else {
            Ok(())
        }
    }

    /// Write the Status-Line and headers of the response, in preparation for writing the body.
    ///
    /// This also overrides the value of the Transfer-Encoding header
    /// (``self.headers.transfer_encoding``), ensuring it is ``None`` if the Content-Length header
    /// has been specified, or to ``chunked`` if it has not, thus switching to the chunked coding.
    ///
    /// If the headers have already been written, this will fail. See also `try_write_headers`.
    pub fn write_headers(&mut self) -> IoResult<()> {
        // This marks the beginning of the response (RFC2616 §6)
        if self.headers_written {
            fail!("ResponseWriter.write_headers() called, but headers already written");
        }

        // Write the Status-Line (RFC2616 §6.1)
        // XXX: might be better not to hardcode HTTP/1.1.
        // XXX: Rust's current lack of statement-duration lifetime handling prevents this from being
        // one statement ("error: borrowed value does not live long enough")
        let s = format!("HTTP/1.1 {}\r\n", self.status.to_str());
        try!(self.writer.write(s.as_bytes()));

        // FIXME: this is not an impressive way of handling it, but so long as chunked is the only
        // transfer-coding we want to deal with it's tolerable. However, it is *meant* to be an
        // extensible thing, whereby client and server could agree upon extra transformations to
        // apply. In such a case, chunked MUST come last. This way prevents it from being extensible
        // thus, which is suboptimal.
        if self.headers.content_length == None {
            self.headers.transfer_encoding = Some(vec!(Chunked));
        } else {
            self.headers.transfer_encoding = None;
        }
        try!(self.headers.write_all(&mut *self.writer));
        self.headers_written = true;
        if self.headers.content_length == None {
            // Flush so that the chunked body stuff can start working correctly. TODO: don't
            // actually flush it entirely, or else it'll send the headers in a separate TCP packet,
            // which is bad for performance.
            try!(self.writer.flush());
            self.writer.writing_chunked_body = true;
        }
        Ok(())
    }

    pub fn finish_response(&mut self) -> IoResult<()> {
        try!(self.writer.finish_response());
        // Ensure that we switch away from chunked in case another request comes on the same socket
        self.writer.writing_chunked_body = false;
        Ok(())
    }
}

impl<'a> Writer for ResponseWriter<'a> {

    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        if !self.headers_written {
            try!(self.write_headers());
        }
        self.writer.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.writer.flush()
    }

}
