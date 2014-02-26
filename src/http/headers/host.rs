//! The Host request header, defined in RFC 2616, Section 14.23.

use std::io::Reader;
use std::fmt;

/// A simple little thing for the host of a request
#[deriving(Clone, Eq)]
pub struct Host {

    /// The name of the host that was requested
    name: ~str,

    /// If unspecified, assume the default port was used (80 for HTTP, 443 for HTTPS).
    /// In that case, you shouldn't need to worry about it in URLs that you build, provided you
    /// include the scheme.
    port: Option<u16>,
}

impl fmt::Show for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.port {
            Some(port) => write!(f.buf, "{}:{}", self.name, port.to_str()),
            None => f.buf.write(self.name.as_bytes()),
        }
    }
}

impl super::HeaderConvertible for Host {
    fn from_stream<R: Reader>(reader: &mut super::HeaderValueByteIterator<R>) -> Option<Host> {
        let s = reader.collect_to_str();
        // TODO: this doesn't support IPv6 address access (e.g. "[::1]")
        // Do this properly with correct authority parsing.
        let mut hi = s.splitn(':', 1);
        Some(Host {
            name: hi.next().unwrap().to_owned(),
            port: match hi.next() {
                Some(name) => from_str::<u16>(name),
                None => None,
            },
        })
    }

    fn http_value(&self) -> ~str {
        self.to_str()
    }
}
