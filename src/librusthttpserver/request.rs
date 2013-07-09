use extra::net::{tcp,ip};
use hashmap = core::hashmap::linear;
use methods::Method;
mod methods;

/// A HTTP request.
///
/// * `host`: The originating IP of the request.
/// * `headers`: The headers of the request.
/// * `body`: The body of the request as a string.
/// * `method`: The method of the request.
/// * `path`: The path of the request
/// * `close_connection`: If the connection should be closed. (Or kept open waiting for more requests)
/// * `version_number`: The HTTP version
pub struct Request {
    host: ip::IpAddr,
    headers: hashmap::LinearMap<~str, ~str>,
    body: ~str,
    method: Method,
    path: ~str,
    close_connection: bool,
    version_number: (int, int)
}

/// Parse an HTTP request line into its parts.
///
/// `parse_request_line("GET /foo HTTP/1.1") == Ok((methods::GET, "/foo", (1, 1)))`
fn parse_request_line(line ~str) -> Option((Method, str, (uint, uint))) {
    // TODO: this probably isn't compliant
    let words = line.word_iter().collect();
    if words.len() != 3 {
        return None;
    }
    let method = Method::from_str_or_new(words[0]);
    let path = words[1]
    let http_version = parse_http_version(words[2]);
    match http_version {
        None => None,
        Some(v) => (method, path, v),
    }
}

/// Parse an HTTP version string into the two X.Y parts.
///
/// At present, HTTP versions the server does not know about are rejected.
///
/// ~~~ {.rust}
/// assert_eq!(parse_http_version(~"HTTP/1.0"), Some((1, 0)))
/// assert_eq!(parse_http_version(~"HTTP/1.1"), Some((1, 1)))
/// assert_eq!(parse_http_version(~"HTTP/2.0"), Some((2, 0)))
/// ~~~
fn parse_http_version(version ~str) -> Option((u8, u8)) {
    match version {
        "HTTP/1.0" => Some(1, 0)
        "HTTP/1.1" => Some(1, 1)
        v if v.starts_with("HTTP/") => {
            let numbers = v.slice_from(5);
            // This would fail! if given non-integers
            //let ints: ~[u8] = v.slice_from(5).split_iter('.').map(|&num| u8::from_str_radix(num, 10).get()).collect();
            let mut ints = [u8 * 2];
            for v.slice_from(5).split_iter('.').enumerate().advance |(i, strnum)| {
                if i > 1 {
                    // More than two numbers, e.g. HTTP/1.2.3
                    return None;
                }
                match u8::from_str_radix(strnum, 10) {
                    Some(num) => ints[i] = num;
                    None => return None;
                }
            }
            (ints[0], ints[1])
        }
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_version() {
        assert_eq!(parse_http_version(~"HTTP/1.1"), Ok((1, 1)));
        assert_eq!(parse_http_version(~"HTTP/1.0"), Ok((1, 0)));
        assert_eq!(parse_http_version(~"HTTP/2.0"), Err(()));
    }

    #[test]
    fn test_parse_request_line() {
        assert_eq!(parse_request_line(~"GET /foo HTTP/1.1"), Ok((methods::GET, "/foo", (1, 1))));
        assert_eq!(parse_request_line(~"POST / HTTP/1.0"), Ok((methods::POST, "/", (1, 0))));
        assert_eq!(parse_request_line(~"POST / HTTP/2.0"), Err(());
    }
}

impl Request {

    /// Get a response from an open socket.
    pub fn get(socket: &tcp::TcpSocket) -> Request {
        let request = socket.read(0u);
        if request.is_err() {
            fail!(~"Bad connection!");
        }
        let request = str::from_bytes(request.get());
    
        let mut lines = ~[];
        for str::each_line_any(request)|line|{lines.push(line)}
        debug!("%?", lines);
        let mut words = ~[];
        for str::each_word(lines.remove(0))|word|{words.push(word)}
    
        let (command, path, close_connection, version_number) = match words.len() {
            3 => {
                let version = words[2];
                if version.slice(0,5) != "HTTP/" {
                    // TODO: send 400
                    fail!(fmt!("Bad request version %?", version))
                }
                let base_version_number = version.slice(5,version.len());
                let mut version_number = ~[];
                for str::each_split_char(base_version_number, '.')|num|{version_number.push(num)}
                if version_number.len() != 2 {
                    // TODO: send 400
                    fail!(fmt!("Bad request version %?", version))
                }
                let version_number = (
                    int::from_str(version_number[0]).unwrap(),
                    int::from_str(version_number[1]).unwrap());
    
                let close_connection = if version_number >= (1, 1) { false } else { true };
                if version_number >= (2, 0) {
                    // TODO: send 505
                    fail!(fmt!("Invalid HTTP Version %?", base_version_number));
                }
                (words[0], words[1], close_connection, version_number)
            },
            2 => {
                let command = words[0];
                if command != "GET" {
                    // TODO: send 400
                    fail!(fmt!("Bad HTTP/0.9 request type %?", command));
                }
                (command, words[1], true, (0 as int, 9 as int))
            },
            _ => {
                // TODO: send 400
                fail!(fmt!("Bad HTTP request words: %?", words));
            }};
    
        let mut headers = hashmap::LinearMap::new::<~str, ~str>();
    
        loop {
            let line = lines.remove(0);
            if (line == ~"\r\n") | (line == ~"\n") | (line == ~"") {break;}
            match str::find_char(line, ':') {
                Some(pos) => {headers.insert(line.slice(0,pos).to_owned(), line.slice(pos+2, line.len()).to_owned());},
                None      => {break;}
            }
        }
    
    
        let close_connection = match headers.find(&~"Connection").unwrap().to_lower() {
            ~"close" => true,
            ~"keep-alive" => false,
            _ => close_connection
        };
    
        Request{
            host: socket.get_peer_addr(),
            headers: headers,
            body: str::connect_slices(lines, "\r\n"),
            method: Method::from_str(command).get(),
            path: path.to_owned(),
            close_connection: close_connection,
            version_number: version_number}
    }
}




pub struct Request {
    // GET, POST, etc.
    method: ~Method,

    // The URL requested, constructed from the request line and (if available)
    // the Host header.
    url: ~Url

    // The HTTP protocol version used; typically (1, 1)
    protocol: (uint, uint)

    // Request headers, all nicely and correctly parsed.
    headers: ~Headers

    // A header maps request lines to their values.
    // If the header says
    //
    //	accept-encoding: gzip, deflate
    //	Accept-Language: en-us
    //	Connection: keep-alive
    //
    // then
    //
    //	Header = map[string][]string{
    //		"Accept-Encoding": {"gzip, deflate"},
    //		"Accept-Language": {"en-us"},
    //		"Connection": {"keep-alive"},
    //	}
    //
    // HTTP defines that header names are case-insensitive.
    // The request parser implements this by canonicalizing the
    // name, making the first character and any characters
    // following a hyphen uppercase and the rest lowercase.
    Header Header

    // The message body.
    Body io.ReadCloser

    // ContentLength records the length of the associated content.
    // The value -1 indicates that the length is unknown.
    // Values >= 0 indicate that the given number of bytes may
    // be read from Body.
    // For outgoing requests, a value of 0 means unknown if Body is not nil.
    ContentLength int64

    // TransferEncoding lists the transfer encodings from outermost to
    // innermost. An empty list denotes the "identity" encoding.
    // TransferEncoding can usually be ignored; chunked encoding is
    // automatically added and removed as necessary when sending and
    // receiving requests.
    TransferEncoding []string

    // Close indicates whether to close the connection after
    // replying to this request.
    Close bool

    // The host on which the URL is sought.
    // Per RFC 2616, this is either the value of the Host: header
    // or the host name given in the URL itself.
    // It may be of the form "host:port".
    Host string

    // Form contains the parsed form data, including both the URL
    // field's query parameters and the POST or PUT form data.
    // This field is only available after ParseForm is called.
    // The HTTP client ignores Form and uses Body instead.
    Form url.Values

    // PostForm contains the parsed form data from POST or PUT
    // body parameters.
    // This field is only available after ParseForm is called.
    // The HTTP client ignores PostForm and uses Body instead.
    PostForm url.Values

    // MultipartForm is the parsed multipart form, including file uploads.
    // This field is only available after ParseMultipartForm is called.
    // The HTTP client ignores MultipartForm and uses Body instead.
    MultipartForm *multipart.Form

    // Trailer maps trailer keys to values.  Like for Header, if the
    // response has multiple trailer lines with the same key, they will be
    // concatenated, delimited by commas.
    // For server requests, Trailer is only populated after Body has been
    // closed or fully consumed.
    // Trailer support is only partially complete.
    Trailer Header

    // RemoteAddr allows HTTP servers and other software to record
    // the network address that sent the request, usually for
    // logging. This field is not filled in by ReadRequest and
    // has no defined format. The HTTP server in this package
    // sets RemoteAddr to an "IP:port" address before invoking a
    // handler.
    // This field is ignored by the HTTP client.
    RemoteAddr string

    // RequestURI is the unmodified Request-URI of the
    // Request-Line (RFC 2616, Section 5.1) as sent by the client
    // to a server. Usually the URL field should be used instead.
    // It is an error to set this field in an HTTP client request.
    RequestURI string

    // TLS allows HTTP servers and other software to record
    // information about the TLS connection on which the request
    // was received. This field is not filled in by ReadRequest.
    // The HTTP server in this package sets the field for
    // TLS-enabled connections before invoking a handler;
    // otherwise it leaves the field nil.
    // This field is ignored by the HTTP client.
    TLS *tls.ConnectionState
}
