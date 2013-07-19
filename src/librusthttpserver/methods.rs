/**
 * An HTTP method.
 */

pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Options,
    Connect,
    Patch,
    UnregisteredMethod(~str),
}

impl ToStr for Method {
    fn to_str(&self) -> ~str {
        match *self {
            Get                       => ~"GET",
            Head                      => ~"HEAD",
            Post                      => ~"POST",
            Put                       => ~"PUT",
            Delete                    => ~"DELETE",
            Trace                     => ~"TRACE",
            Options                   => ~"OPTIONS",
            Connect                   => ~"CONNECT",
            Patch                     => ~"PATCH",
            UnregisteredMethod(ref s) => (*s).clone(),
        }
    }
}

/** Normalise an HTTP method name into capital letters.
 *
 * IMPORTANT: ensure that method.is_ascii() is true before calling this,
 * or things will probably blow up in an uncomfortable sort of way
 */
fn normalise_method_name(method: &str) -> ~str {
    unsafe { method.to_ascii_nocheck() }.to_upper().to_str_ascii()
}

impl FromStr for Method {
    /**
     * Get a *known* `Method` from an *ASCII* string, regardless of case.
     *
     * If you want to support unregistered methods, use `from_str_or_new` instead.
     *
     * (If the string isn't ASCII, this will at present fail: TODO fix that.)
     */
    pub fn from_str(method: &str) -> Option<Method> {
        if (!method.is_ascii()) {
            return None;
        }
        match normalise_method_name(method) {
            ~"GET"     => Some(Get),
            ~"HEAD"    => Some(Head),
            ~"POST"    => Some(Post),
            ~"PUT"     => Some(Put),
            ~"DELETE"  => Some(Delete),
            ~"TRACE"   => Some(Trace),
            ~"OPTIONS" => Some(Options),
            ~"CONNECT" => Some(Connect),
            ~"PATCH"   => Some(Patch),
            _          => None
        }
    }
}

impl Method {
    /**
     * Get a `Method` from an *ASCII* string.
     *
     * (If the string isn't ASCII, this will at present fail.)
     */
    pub fn from_str_or_new(method: &str) -> Method {
        assert!(method.is_ascii());
        match normalise_method_name(method) {
            ~"GET"     => Get,
            ~"HEAD"    => Head,
            ~"POST"    => Post,
            ~"PUT"     => Put,
            ~"DELETE"  => Delete,
            ~"TRACE"   => Trace,
            ~"OPTIONS" => Options,
            ~"CONNECT" => Connect,
            ~"PATCH"   => Patch,
            _          => UnregisteredMethod(method.to_owned()),
        }
    }
}
