//! Types and utilities for working with headers in HTTP requests and responses.
//!
//! Notably, HTTP headers are case insensitive; headers are represented as a `TreeMap`, and so there
//! is, at present, scope for mistakes in providing multiple values for a header under different
//! cases. You should, of course, avoid this.
//!
//! Headers can be normalised into the canonical case employed in this library with
//! `normalise_header_name`; that function defines the canonical case. Notably, this will mean that
//! the naming conventions are *not* followed for certain headers: `Content-MD5` would be sent as
//! `Content-Md5`, `TE` as `Te` and `WWW-Authenticate` as `Www-Authenticate`.
//!
//! Another common convention among HTTP headers is to use comma-separated values,
//! e.g. `Accept: text/html, text/plain;q=0.8, text/*;q=0.1`. For transforming to and from these
//! values we have `comma_split` and `comma_join`.

use std::vec;
use std::ascii::Ascii;
use extra::treemap::TreeMap;

/// Normalise an HTTP header name.
///
/// Rules:
///
/// - The first character is capitalised
/// - Any character immediately following `-` (HYPHEN-MINUS) is capitalised
/// - All other characters are made lowercase
///
/// This will fail if passed a non-ASCII name.
///
/// # Examples
///
/// ~~~ .{rust}
/// assert_eq!(normalise_header_name("foo-bar"), "Foo-Bar");
/// assert_eq!(normalise_header_name("FOO-BAR"), "Foo-Bar");
/// ~~~
pub fn normalise_header_name(name: &str) -> ~str {
    let mut result: ~[Ascii] = vec::with_capacity(name.len());
    let mut capitalise = true;
    for name.iter().advance |c| {
        let c = match capitalise {
            true => c.to_ascii().to_upper(),
            false => c.to_ascii().to_lower(),
        };
        result.push(c);
        // ASCII 45 is '-': in that case, capitalise the next char
        capitalise = c.to_byte() == 45;
    }
    result.to_str_ascii()
}

/// Split a value on commas, as is common for HTTP headers.
///
/// # Examples
///
/// ~~~ .{rust}
/// assert_eq!(comma_split(" en;q=0.8, en_AU, text/html"), ["en;q=0.8", "en_AU", "text/html"])
/// ~~~
pub fn comma_split(value: &str) -> ~[~str] {
    value.split_iter(',').transform(|w| w.trim_left().to_owned()).collect()
}

/// Join a vector of values with commas, as is common for HTTP headers.
///
/// # Examples
///
/// ~~~ .{rust}
/// assert_eq!(comma_split(["en;q=0.8", "en_AU", "text/html"]), "en;q=0.8, en_AU, text/html")
/// ~~~
#[inline]
pub fn comma_join(values: &[&str]) -> ~str {
    values.connect(", ")
}

pub type Headers = TreeMap<~str, ~str>;

/*impl Headers {
    fn new() -> ~Headers {
        ~Headers(*TreeMap::new::<~str, ~str>())
    }
}*/

/* In the interests simplicity of implementation, I think we'll leave the automatic normalisation
 * out for the present. Theoretically better for performance, too, unless it causes mistakes...

/// Headers
pub struct Headers {
    priv map: TreeMap<~str, ~[~str]>,
}

impl Headers {

    /// Get the named header
    pub fn get(&self, name: &str) -> Option<~str> {
        //let name = normalise_header_name(name);
        let mut concatenated = "";
        for map.find(name).iter().advance |hunk| {
            concatenated += fmt!(", %s", hunk);
        }
        concatenated
    }

    /// Get the named header, split by commas
    pub fn get_vec(&self, name: &str) -> Option<~[~str]> {
        //let name = normalise_header_name(name);
        self.map.find(name)
    }

    pub fn get_vec_mut(&mut self, name: &str) -> Option<~[~str]> {
        //let name = normalise_header_name(name);
        self.map.find_mut(name)
    }

    pub fn set(&self, name: &str, value: &str) {
        //let name = normalise_header_name(name);
        // TODO: improve this; it's probably overly simplistic
        let values: ~[~str] = value.split_iter(',').skip_while(|c| c == ' ').collect();
        self.map.insert(name, values);
    }

    pub fn set(&self, name: &str, values: &[&str]) {
        //let name = normalise_header_name(name);
        // TODO: improve this; it's probably overly simplistic
        self.map.insert(name, values);
    }

    pub fn iter<'a>(&'a self) -> TreeMapIterator<'a, K, V> {
        self.map.iter().transform(|(name, values)| {
            let mut concatonated = "";
            for values.iter().advance |hunk| {
                concatenated += fmt!(", %s", hunk);
            }
            concatenated
        })
    }

    pub fn iter_vec<'a>(&'a self) -> TreeMapIterator<'a, K, V> {
        self.map.iter()
    }
}
*/

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_fail]
    fn test_normalise_header_name_fail() {
        normalise_header_name("foö-bar");
    }

    #[test]
    fn test_normalise_header_name() {
        assert_eq!(normalise_header_name("foo-bar"), ~"Foo-Bar");
        assert_eq!(normalise_header_name("FOO-BAR"), ~"Foo-Bar");
    }

    #[test]
    fn test_comma_split() {
        assert_eq!(comma_split("foo"), ~[~"foo"]);
        assert_eq!(comma_split("foo,bar"), ~[~"foo", ~"bar"]);
        assert_eq!(comma_split(" foo;q=0.8 , bar/* "), ~[~"foo;q=0.8 ", ~"bar/* "]);
    }

    #[test]
    fn test_comma_join() {
        assert_eq!(comma_join(["foo"]), ~"foo");
        assert_eq!(comma_join(["foo", "bar"]), ~"foo, bar");
        assert_eq!(comma_join([" foo;q=0.8 ", "bar/* "]), ~" foo;q=0.8 , bar/* ");
    }
}

/* A could-be-nice: have getter and setter methods for each (or most) of these, doing the
 * appropriate type conversion: (c.f. https://en.wikipedia.org/wiki/List_of_HTTP_headers)
Access-Control-Allow-Origin
Accept-Ranges
Age
Allow
Cache-Control
Connection
Content-Encoding
Content-Language
Content-Length
Content-Location
Content-MD5
Content-Disposition
Content-Range
Content-Type
Date
ETag
Expires
Last-Modified
Link
Location
P3P
Pragma
Proxy-Authenticate
Refresh
Retry-After
Server
Set-Cookie
Status
Strict-Transport-Security
Trailer
Transfer-Encoding
Vary
Via
Warning
WWW-Authenticate

// Common non-standard ones
X-Frame-Options
X-XSS-Protection
Content-Security-Policy, X-Content-Security-Policy, X-WebKit-CSP
X-Content-Type-Options
X-Powered-By
X-UA-Compatible
*/
