// These are taken from http://en.wikipedia.org/wiki/List_of_HTTP_Status_Codes.
// Last updated on 2013-07-04
// Entries from third-party vendors not standardised upon are not included.
// If not specified otherwise, they are defined in RFC 2616.

// Yes, this is ugly code.
// No, I don't mind.
// That was easy. :-)

use collections::hashmap::HashSet;
use std::ascii::StrAsciiExt;
use std::vec;
use std::io::IoResult;
use super::get_writer;

enum HeadingOrStatus {
    Heading(&'static str),
    Status(Status),
}

struct Status {
    code: uint,
    reason: &'static str,
    comment: Option<&'static str>,
}

/// Status with comment
fn StatusC(code: uint, reason: &'static str, comment: &'static str) -> HeadingOrStatus {
    Status(Status { code: code, reason: reason, comment: Some(comment) })
}

/// Status without comment
fn StatusN(code: uint, reason: &'static str) -> HeadingOrStatus {
    Status(Status { code: code, reason: reason, comment: None })
}

impl Status {
    fn ident(&self) -> ~str {
        camel_case(self.reason)
    }

    fn padded_ident(&self) -> ~str {
        self.ident() + " ".repeat(unsafe { longest_ident } - self.ident().len())
    }

    fn reason_padding_spaces(&self) -> ~str {
        " ".repeat(unsafe { longest_reason } - self.reason.len())
    }
}

/// >>> camel_case("I'm a Tea-pot")
/// "ImATeaPot"
fn camel_case(msg: &str) -> ~str {
    let msg = msg.replace("-", " ").replace("'", "");
    let mut result: ~[Ascii] = vec::with_capacity(msg.len());
    let mut capitalise = true;
    for c in msg.chars() {
        let c = match capitalise {
            true => c.to_ascii().to_upper(),
            false => c.to_ascii().to_lower(),
        };
        // For a space, capitalise the next char
        capitalise = c.to_byte() == (' ' as u8);
        if !capitalise {  // And also, for a space, don't keep it
            result.push(c);
        }
    }
    result.into_str()
}

static mut longest_ident: uint = 0;
static mut longest_reason: uint = 0;

pub fn generate(output_dir: &Path) -> IoResult<()> {
    let mut out = get_writer(output_dir, "status.rs");
    let entries = [
        Heading("1xx Informational"),
        StatusN(100, "Continue"),
        StatusN(101, "Switching Protocols"),
        StatusC(102, "Processing", "WebDAV; RFC 2518"),

        Heading("2xx Success"),
        StatusN(200, "OK"),
        StatusN(201, "Created"),
        StatusN(202, "Accepted"),
        StatusC(203, "Non-Authoritative Information", "since HTTP/1.1"),
        StatusN(204, "No Content"),
        StatusN(205, "Reset Content"),
        StatusN(206, "Partial Content"),
        StatusC(207, "Multi-Status", "WebDAV; RFC 4918"),
        StatusC(208, "Already Reported", "WebDAV; RFC 5842"),
        StatusC(226, "IM Used", "RFC 3229"),

        Heading("3xx Redirection"),
        StatusN(300, "Multiple Choices"),
        StatusN(301, "Moved Permanently"),
        StatusN(302, "Found"),
        StatusC(303, "See Other", "since HTTP/1.1"),
        StatusN(304, "Not Modified"),
        StatusC(305, "Use Proxy", "since HTTP/1.1"),
        StatusN(306, "Switch Proxy"),
        StatusC(307, "Temporary Redirect", "since HTTP/1.1"),
        StatusC(308, "Permanent Redirect", "approved as experimental RFC: http://tools.ietf.org/html/draft-reschke-http-status-308"),

        Heading("4xx Client Error"),
        StatusN(400, "Bad Request"),
        StatusN(401, "Unauthorized"),
        StatusN(402, "Payment Required"),
        StatusN(403, "Forbidden"),
        StatusN(404, "Not Found"),
        StatusN(405, "Method Not Allowed"),
        StatusN(406, "Not Acceptable"),
        StatusN(407, "Proxy Authentication Required"),
        StatusN(408, "Request Timeout"),
        StatusN(409, "Conflict"),
        StatusN(410, "Gone"),
        StatusN(411, "Length Required"),
        StatusN(412, "Precondition Failed"),
        StatusN(413, "Request Entity Too Large"),
        StatusN(414, "Request-URI Too Long"),
        StatusN(415, "Unsupported Media Type"),
        StatusN(416, "Requested Range Not Satisfiable"),
        StatusN(417, "Expectation Failed"),
        StatusC(418, "I'm a teapot", "RFC 2324"),
        StatusN(419, "Authentication Timeout"),
        StatusC(422, "Unprocessable Entity", "WebDAV; RFC 4918"),
        StatusC(423, "Locked", "WebDAV; RFC 4918"),
        StatusC(424, "Failed Dependency", "WebDAV; RFC 4918"),
        StatusC(424, "Method Failure", "WebDAV"),
        StatusC(425, "Unordered Collection", "Internet draft"),
        StatusC(426, "Upgrade Required", "RFC 2817"),
        StatusC(428, "Precondition Required", "RFC 6585"),
        StatusC(429, "Too Many Requests", "RFC 6585"),
        StatusC(431, "Request Header Fields Too Large", "RFC 6585"),
        StatusC(451, "Unavailable For Legal Reasons", "Internet draft"),

        Heading("5xx Server Error"),
        StatusN(500, "Internal Server Error"),
        StatusN(501, "Not Implemented"),
        StatusN(502, "Bad Gateway"),
        StatusN(503, "Service Unavailable"),
        StatusN(504, "Gateway Timeout"),
        StatusN(505, "HTTP Version Not Supported"),
        StatusC(506, "Variant Also Negotiates", "RFC 2295"),
        StatusC(507, "Insufficient Storage", "WebDAV; RFC 4918"),
        StatusC(508, "Loop Detected", "WebDAV; RFC 5842"),
        StatusC(510, "Not Extended", "RFC 2774"),
        StatusC(511, "Network Authentication Required", "RFC 6585"),
        ];
    unsafe {
        longest_ident = entries.iter().map(|&e| match e {
            Heading(_heading) => 0,
            Status(status) => status.ident().len(),
        }).max_by(|&i| i).unwrap();
        longest_reason = entries.iter().map(|&e| match e {
            Heading(_heading) => 0,
            Status(status) => status.reason.len(),
        }).max_by(|&i| i).unwrap();
    }
    try!(out.write("// This file is automatically generated file is used as http::status.

use std::fmt;
use std::ascii::StrAsciiExt;

/// HTTP status code
#[deriving(Eq, Clone)]
pub enum Status {
".as_bytes()));
    for &entry in entries.iter() {
        match entry {
            Heading(heading) => try!(write!(out, "\n    // {}\n", heading)),
            Status(status) => match status.comment {
                None => try!(write!(out, "    {},\n", status.ident())),
                Some(comment) => try!(write!(out, "    {},  // {}\n", status.ident(), comment)),
            },
        }
    }

    try!(out.write("
    UnregisteredStatus(u16, ~str),
}

impl Status {

    /// Get the status code
    pub fn code(&self) -> u16 {
        match *self {
".as_bytes()));
    for &entry in entries.iter() {
        match entry {
            Heading(heading) => try!(write!(out, "\n            // {}\n", heading)),
            Status(status) => try!(write!(out, "            {} => {},\n",
                                                status.padded_ident(), status.code)),
        }
    }
    try!(out.write("
            UnregisteredStatus(code, _)   => code,
        }
    }

    /// Get the reason phrase
    pub fn reason(&self) -> ~str {
        match *self {
".as_bytes()));
    for &entry in entries.iter() {
        match entry {
            Heading(heading) => try!(write!(out, "\n            // {}\n", heading)),
            Status(status) => try!(write!(out, "            {} => ~\"{}\",\n",
                                                status.padded_ident(), status.reason))
        }
    }
    try!(out.write("
            UnregisteredStatus(_, ref reason) => (*reason).clone(),
        }
    }

    /// Get a status from the code and reason
    pub fn from_code_and_reason(status: u16, reason: ~str) -> Status {
        let reason_lower = reason.to_ascii_lower();
        match (status, reason_lower.as_slice()) {
".as_bytes()));
    for &entry in entries.iter() {
        match entry {
            Heading(heading) => try!(write!(out, "\n            // {}\n", heading)),
            Status(status) => try!(write!(out, "            ({}, \"{}\"){} => {},\n",
                                                status.code,
                                                status.reason.to_ascii_lower(),
                                                status.reason_padding_spaces(),
                                                status.ident())),
        }
    }
    try!(out.write("
            (_, _) => UnregisteredStatus(status, reason),
        }
    }
}

impl fmt::Show for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, \"{} {}\", self.code(), self.reason())
    }
}

impl fmt::Unsigned for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Unsigned;
        self.code().fmt(f)
    }
}


impl ToPrimitive for Status {

    /// Equivalent to `Some(self.code() as i64)`
    fn to_i64(&self) -> Option<i64> {
        Some(self.code() as i64)
    }

    /// Equivalent to `Some(self.code() as u64)`
    fn to_u64(&self) -> Option<u64> {
        Some(self.code() as u64)
    }
}

impl FromPrimitive for Status {
    /// Get a *registered* status code from the number of its status code.
    ///
    /// This will return None if an unknown (or negative, which are invalid) code is passed.
    ///
    /// For example, `from_i64(200)` will return `OK`.
    fn from_i64(n: i64) -> Option<Status> {
        if n < 0 {
            None
        } else {
            FromPrimitive::from_u64(n as u64)
        }
    }

    /// Get a *registered* status code from the number of its status code.
    ///
    /// This will return None if an unknown code is passed.
    ///
    /// For example, `from_u64(200)` will return `OK`.
    fn from_u64(n: u64) -> Option<Status> {
        Some(match n {
".as_bytes()));
    let mut matched_numbers = HashSet::new();
    for &entry in entries.iter() {
        match entry {
            Heading(heading) => try!(write!(out, "\n            // {}\n", heading)),
            Status(status) => {
                if !matched_numbers.contains(&status.code) {
                    // Purpose: FailedDependency and MethodFailure both use 424,
                    // but clearly they mustn't both go in here
                    try!(write!(out, "            {:u} => {},\n", status.code, status.ident()));
                    matched_numbers.insert(status.code);
                }
            },
        }
    }
    out.write("
            _   => { return None }
        })
    }
}".as_bytes())
}
