use std::fmt;
use std::from_str::FromStr;

/// HTTP methods, as defined in RFC 2616, §5.1.1.
///
/// Method names are case-sensitive.
#[deriving(Eq,Clone)]
pub enum Method {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
    Patch,  // RFC 5789
    ExtensionMethod(~str),
}

impl FromStr for Method {
    /**
     * Get a *known* `Method` from an *ASCII* string, regardless of case.
     *
     * If you want to support unregistered methods, use `from_str_or_new` instead.
     *
     * (If the string isn't ASCII, this will at present fail: TODO fix that.)
     */
    fn from_str(method: &str) -> Option<Method> {
        if !method.is_ascii() {
            return None;
        }
        match method {
            "OPTIONS" => Some(Options),
            "GET"     => Some(Get),
            "HEAD"    => Some(Head),
            "POST"    => Some(Post),
            "PUT"     => Some(Put),
            "DELETE"  => Some(Delete),
            "TRACE"   => Some(Trace),
            "CONNECT" => Some(Connect),
            "PATCH"   => Some(Patch),
            _         => None
        }
    }
}

impl fmt::Show for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.buf.write(match *self {
            Options                => "OPTIONS".as_bytes(),
            Get                    => "GET".as_bytes(),
            Head                   => "HEAD".as_bytes(),
            Post                   => "POST".as_bytes(),
            Put                    => "PUT".as_bytes(),
            Delete                 => "DELETE".as_bytes(),
            Trace                  => "TRACE".as_bytes(),
            Connect                => "CONNECT".as_bytes(),
            Patch                  => "PATCH".as_bytes(),
            ExtensionMethod(ref s) => s.as_bytes(),
        })
    }
}

impl Method {
    /**
     * Get a `Method` from an *ASCII* string.
     *
     * (If the string isn't ASCII, this will at present fail.)
     */
    pub fn from_str_or_new(method: &str) -> Option<Method> {
        assert!(method.is_ascii());
        Some(match method {
            "OPTIONS" => Options,
            "GET"     => Get,
            "HEAD"    => Head,
            "POST"    => Post,
            "PUT"     => Put,
            "DELETE"  => Delete,
            "TRACE"   => Trace,
            "CONNECT" => Connect,
            "PATCH"   => Patch,
            _         => ExtensionMethod(method.to_owned()),
        })
    }
}
