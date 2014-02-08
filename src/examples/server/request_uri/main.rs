//! An HTTP server demonstrating the probable direction of the library without actually being *in*
//! the library.
//!
//! This demonstrates some handling of the RequestURI, which has several possibilities and for which
//! the correct values depend on the method.

#[crate_id = "request_uri"];

extern mod extra;
extern mod http;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;
use extra::time;

use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{Star, AbsoluteUri, AbsolutePath, Authority};
use http::status::{BadRequest, MethodNotAllowed};
use http::method::{Get, Head, Post, Put, Delete, Trace, Options, Connect, Patch};
use http::headers::content_type::MediaType;

#[deriving(Clone)]
struct RequestUriServer;

impl Server for RequestUriServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8001 } }
    }

    fn handle_request(&self, r: &Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.server = Some(~"Rust Thingummy/0.1-pre");

        match (&r.method, &r.request_uri) {
            (&Connect, _) => {
                // "This specification reserves the method name CONNECT for use with a proxy that
                // can dynamically switch to being a tunnel (e.g. SSL tunneling Tunneling TCP based
                // protocols through Web proxy servers)." Thus, not applicable.
                w.status = MethodNotAllowed;
                return
            },
            (_, &Authority(_)) => {
                // "The authority form is only used by the CONNECT method." Thus, not applicable.
                w.status = BadRequest;
                return
            },
            (&Options, &Star) => {
                // Querying server capabilities. That's nice and simple. I can handle these methods:
                // (TODO: let user code override this, providing a default method.)
                w.headers.allow = Some(~[Get, Head, Post, Put, Delete, Trace, Options, Connect, Patch]);
                w.headers.content_length = Some(0);
                return;
            },
            (&Options, &AbsoluteUri(_)) | (&Options, &AbsolutePath(_)) => {
            },
            (_, &AbsoluteUri(_)) | (_, &AbsolutePath(_)) => {
            },
            (_, &Star) => {
            },
        }

        w.headers.content_type = Some(MediaType {
            type_: ~"text",
            subtype: ~"html",
            parameters: ~[]
        });

        w.write(bytes!("<!DOCTYPE html><title>Rust HTTP server</title>")).unwrap();

        match r.request_uri {
            Star | Authority(_) => {
                w.status = BadRequest;
                // Actually, valid for the CONNECT method.
            },
            AbsoluteUri(ref url) => {
                println!("absoluteURI, {}", url.to_str());
                //path = 
            },
            AbsolutePath(ref url) => {
                println!("absolute path, {}", url.to_owned());
                //w.status = a
            },
        }
    }
}

fn main() {
    RequestUriServer.serve_forever();
}
