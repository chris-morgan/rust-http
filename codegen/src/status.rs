// These are taken from http://en.wikipedia.org/wiki/List_of_HTTP_Status_Codes.
// Last updated on 2013-07-04
// Entries from third-party vendors not standardised upon are not included.
// If not specified otherwise, they are defined in RFC 2616.

// Yes, this is ugly code.
// No, I don't mind.
// That was easy. :-)

use std::collections::hashmap::HashSet;
use std::ascii::StrAsciiExt;
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
fn status_c(code: uint, reason: &'static str, comment: &'static str) -> HeadingOrStatus {
    Status(Status { code: code, reason: reason, comment: Some(comment) })
}

/// Status without comment
fn status_n(code: uint, reason: &'static str) -> HeadingOrStatus {
    Status(Status { code: code, reason: reason, comment: None })
}

impl Status {
    fn ident(&self) -> String {
        camel_case(self.reason)
    }

    fn padded_ident(&self) -> String {
        self.ident().append(self.reason_padding_spaces().as_slice())
    }

    fn reason_padding_spaces(&self) -> String {
        " ".repeat(unsafe { longest_reason } - self.reason.len())
    }
}

/// >>> camel_case("I'm a Tea-pot")
/// "ImATeaPot"
fn camel_case(msg: &str) -> String {
    let msg = msg.replace("-", " ").replace("'", "");
    let mut result = String::with_capacity(msg.len());
    let mut capitalise = true;
    for c in msg.as_slice().chars() {
        let c = match capitalise {
            true => c.to_ascii().to_uppercase().to_char(),
            false => c.to_ascii().to_lowercase().to_char(),
        };
        // For a space, capitalise the next char
        capitalise = c == ' ';
        if !capitalise {  // And also, for a space, don't keep it
            result.push_char(c);
        }
    }
    result
}

static mut longest_ident: uint = 0;
static mut longest_reason: uint = 0;

pub fn generate(output_dir: &Path) -> IoResult<()> {
    let mut out = get_writer(output_dir, "status.rs");
    let entries = [
        Heading("1xx Informational"),
        status_n(100, "Continue"),
        status_n(101, "Switching Protocols"),
        status_c(102, "Processing", "WebDAV; RFC 2518"),

        Heading("2xx Success"),
        status_n(200, "OK"),
        status_n(201, "Created"),
        status_n(202, "Accepted"),
        status_c(203, "Non-Authoritative Information", "since HTTP/1.1"),
        status_n(204, "No Content"),
        status_n(205, "Reset Content"),
        status_n(206, "Partial Content"),
        status_c(207, "Multi-Status", "WebDAV; RFC 4918"),
        status_c(208, "Already Reported", "WebDAV; RFC 5842"),
        status_c(226, "IM Used", "RFC 3229"),

        Heading("3xx Redirection"),
        status_n(300, "Multiple Choices"),
        status_n(301, "Moved Permanently"),
        status_n(302, "Found"),
        status_c(303, "See Other", "since HTTP/1.1"),
        status_n(304, "Not Modified"),
        status_c(305, "Use Proxy", "since HTTP/1.1"),
        status_n(306, "Switch Proxy"),
        status_c(307, "Temporary Redirect", "since HTTP/1.1"),
        status_c(308, "Permanent Redirect", "approved as experimental RFC: http://tools.ietf.org/html/draft-reschke-http-status-308"),

        Heading("4xx Client Error"),
        status_n(400, "Bad Request"),
        status_n(401, "Unauthorized"),
        status_n(402, "Payment Required"),
        status_n(403, "Forbidden"),
        status_n(404, "Not Found"),
        status_n(405, "Method Not Allowed"),
        status_n(406, "Not Acceptable"),
        status_n(407, "Proxy Authentication Required"),
        status_n(408, "Request Timeout"),
        status_n(409, "Conflict"),
        status_n(410, "Gone"),
        status_n(411, "Length Required"),
        status_n(412, "Precondition Failed"),
        status_n(413, "Request Entity Too Large"),
        status_n(414, "Request-URI Too Long"),
        status_n(415, "Unsupported Media Type"),
        status_n(416, "Requested Range Not Satisfiable"),
        status_n(417, "Expectation Failed"),
        status_c(418, "I'm a teapot", "RFC 2324"),
        status_n(419, "Authentication Timeout"),
        status_c(422, "Unprocessable Entity", "WebDAV; RFC 4918"),
        status_c(423, "Locked", "WebDAV; RFC 4918"),
        status_c(424, "Failed Dependency", "WebDAV; RFC 4918"),
        status_c(424, "Method Failure", "WebDAV"),
        status_c(425, "Unordered Collection", "Internet draft"),
        status_c(426, "Upgrade Required", "RFC 2817"),
        status_c(428, "Precondition Required", "RFC 6585"),
        status_c(429, "Too Many Requests", "RFC 6585"),
        status_c(431, "Request Header Fields Too Large", "RFC 6585"),
        status_c(451, "Unavailable For Legal Reasons", "Internet draft"),

        Heading("5xx Server Error"),
        status_n(500, "Internal Server Error"),
        status_n(501, "Not Implemented"),
        status_n(502, "Bad Gateway"),
        status_n(503, "Service Unavailable"),
        status_n(504, "Gateway Timeout"),
        status_n(505, "HTTP Version Not Supported"),
        status_c(506, "Variant Also Negotiates", "RFC 2295"),
        status_c(507, "Insufficient Storage", "WebDAV; RFC 4918"),
        status_c(508, "Loop Detected", "WebDAV; RFC 5842"),
        status_c(510, "Not Extended", "RFC 2774"),
        status_c(511, "Network Authentication Required", "RFC 6585"),
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
#[deriving(Eq, PartialEq, Clone)]
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
    UnregisteredStatus(u16, String),
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
    pub fn reason(&self) -> String {
        match *self {
".as_bytes()));
    for &entry in entries.iter() {
        match entry {
            Heading(heading) => try!(write!(out, "\n            // {}\n", heading)),
            Status(status) => try!(write!(out, "            {} => String::from_str(\"{}\"),\n",
                                                status.padded_ident(), status.reason))
        }
    }
    try!(out.write("
            UnregisteredStatus(_, ref reason) => (*reason).clone(),
        }
    }

    /// Get a status from the code and reason
    pub fn from_code_and_reason(status: u16, reason: String) -> Status {
        let reason_lower = reason.as_slice().to_ascii_lower();
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
        write!(f, \"{} {}\", self.code(), self.reason())
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
