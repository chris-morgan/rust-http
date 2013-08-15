use extra::treemap::TreeMap;
use std::rt::io::Reader;
use std::rt::io::extensions::ReaderUtil;
use std::rt::io::{io_error, OtherIoError, IoError};
use std::rt;
use std::either::{Left, Right};
use client::request::RequestWriter;
use rfc2616::{CR, LF, SP};
use common::read_http_version;
use headers::{Headers, normalise_header_name};
use headers;
use status::Status;

use buffer::BufTcpStream;
use server::request::{RequestBuffer, HeaderLineErr, EndOfFile, EndOfHeaders, MalformedHeader};

struct ResponseReader {
    priv stream: BufTcpStream,

    /// The request which this is a response to
    request: ~RequestWriter,

    /// The HTTP version number; typically `(1, 1)` or, less commonly, `(1, 0)`.
    version: (uint, uint),

    /// The HTTP status indicated in the response.
    status: Status,

    /// The headers received in the response.
    headers: ~Headers,
}

fn bad_response_err() -> IoError {
    // TODO: IoError isn't right
    IoError {
        kind: OtherIoError,
        desc: "Server returned malformed HTTP response",
        detail: None,
    }
}

impl ResponseReader {
    pub fn construct(mut stream: BufTcpStream, request: ~RequestWriter)
            -> Result<ResponseReader, ~RequestWriter> {
        // TODO: raise condition at the points where Err is returned
        //let mut b = [0u8, ..4096];
        //let len = stream.read(b);
        //printfln!("%?", ::std::str::from_bytes(b.slice_to(len.unwrap())));
        let http_version = match read_http_version(&mut stream, SP) {
            Some(nums) => nums,
            None => {
                io_error::cond.raise(bad_response_err());
                return Err(request);
            }
        };

        // Read the status code
        let mut digits = 0u8;
        let mut status_code = 0u16;
        loop {
            if digits == 4u8 {
                // Status code must be three digits long
                io_error::cond.raise(bad_response_err());
                return Err(request);
            }
            match stream.read_byte() {
                Some(b) if b >= '0' as u8 && b <= '9' as u8 => {
                    status_code = status_code * 10 + b as u16 - '0' as u16;
                },
                Some(b) if b == SP => break,
                _ => {
                    io_error::cond.raise(bad_response_err());
                    return Err(request);
                }
            }
            digits += 1;
        }

        // Read the status reason
        let mut reason = ~"";
        loop {
            match stream.read_byte() {
                Some(b) if b == CR => {
                    if stream.read_byte() == Some(LF) {
                        break;
                    } else {
                        // Response-Line has CR without LF. Not yet resilient; TODO.
                        io_error::cond.raise(bad_response_err());
                        return Err(request);
                    }
                }
                Some(b) => {
                    reason.push_char(b as char);
                }
                None => {
                    io_error::cond.raise(bad_response_err());
                    return Err(request);
                }
            }
        }

        // Now we sneakily slip back to server::RequestBuffer to avoid code duplication. This is
        // temporary, honest!
        //
        // You see, read_header and read_header_line will be replaced, as will this. The code will
        // not be shared between them as they will have ultra-smart parsers (probably using Ragel)
        // to provide fast loading of standard headers, and the set of defined headers is distinct
        // between a request and response.
        let headers = {
            let mut buffer = RequestBuffer::new(&mut stream);
            let mut headers = ~TreeMap::new();
            loop {
                match read_header_line(&mut buffer) {
                    Err(Left(EndOfFile)) => {
                        io_error::cond.raise(bad_response_err());
                        //fail!("server disconnected, no more response to receive :-(");
                        return Err(request);
                    },
                    Err(Left(EndOfHeaders)) => break,
                    Err(Left(MalformedHeader)) => {
                        io_error::cond.raise(bad_response_err());
                        return Err(request);
                    },
                    Err(Right(cause)) => {
                        printfln!("Bad header encountered (%s). TODO: handle this better.", cause);
                        // Now just ignore the header
                    },
                    Ok((name, value)) => {
                        headers.insert(normalise_header_name(name), value);
                    },
                }
            }
            headers
        };

        Ok(ResponseReader {
            stream: stream,
            request: request,
            version: http_version,
            status: Status::from_code_and_reason(status_code, reason),
            headers: headers,
        })
    }
}

impl rt::io::Reader for ResponseReader {
    fn read(&mut self, buf: &mut [u8]) -> Option<uint> {
        self.stream.read(buf)
    }

    fn eof(&mut self) -> bool {
        self.stream.eof()
    }
}

/// TODO: kill this too.
pub fn read_header_line(rb: &mut RequestBuffer) -> Result<(~str, ~str),
                                                           Either<HeaderLineErr, &'static str>> {
    match rb.read_header::<headers::response::Header>() {
        Ok(headers::response::ExtensionHeader(k, v)) => Ok((k, v)),
        Ok(h) => {
            printfln!("[31;1mHeader dropped (TODO):[0m %?", h);
            Err(Right("header interpreted but I can't yet use it: dropped"))
        },
        Err(e) => Err(e),
    }
}
