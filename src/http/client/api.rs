use std::default::Default;
use std::io::net::tcp::TcpStream;

use method;
use headers::request::HeaderCollection;

use client::request::RequestWriter;
use client::response::ResponseReader;
use extra::url::{Url, Query};

pub struct RequestArgs {

    // Request data
    data: Option<~[u8]>,

    // Query Parameters
    params: Option<Query>,

    // Request Headers
    headers: Option<~HeaderCollection>,
}

impl Default for RequestArgs {

    fn default() -> RequestArgs {
        RequestArgs{data: None, params: None, headers: None}
    }
}

// Need a fix for https://github.com/mozilla/rust/issues/9056
// before we can use this.
//pub static DEFAULT_ARGS: RequestArgs = RequestArgs{params: None,
//                                                   headers: None};

// TODO: Implement a Response trait

pub fn request(method: method::Method, url: ~str,  args: Option<RequestArgs>) 
    -> Result<ResponseReader<TcpStream>, RequestWriter<TcpStream>>{

    let default = args.unwrap_or_default();

    // Push all query params to the URL.
    let mut url: Url = FromStr::from_str(url).expect("Uh oh, that's *really* badly broken!");

    if default.params.is_some() {
        url.query.push_all(*(default.params.get_ref()));
    }

    // At this point, we're ready to finally send
    // the request. First thing is to write headers,
    // then the request data and later get the response
    // from the server.
    let mut request = RequestWriter::new(method, url);

    // Write data if there's some
    if default.data.is_some() {
        request.send(*(default.data.get_ref()));
    }

    // This will flush the request's
    // stream and get the response from
    // the server.
    request.read_response()
}

pub fn get(url: ~str, args: Option<RequestArgs>)
    -> Result<ResponseReader<TcpStream>, RequestWriter<TcpStream>> {

    request(method::Get, url, args)
}

pub fn post(url: ~str, args: Option<RequestArgs>)
    -> Result<ResponseReader<TcpStream>, RequestWriter<TcpStream>> {

    request(method::Post, url, args)
}

pub fn patch(url: ~str, args: Option<RequestArgs>)
    -> Result<ResponseReader<TcpStream>, RequestWriter<TcpStream>> {

    request(method::Patch, url, args)
}

pub fn put(url: ~str, args: Option<RequestArgs>)
    -> Result<ResponseReader<TcpStream>, RequestWriter<TcpStream>> {

    request(method::Put, url, args)
}

pub fn delete(url: ~str, args: Option<RequestArgs>)
    -> Result<ResponseReader<TcpStream>, RequestWriter<TcpStream>> {

    request(method::Delete, url, args)
}
