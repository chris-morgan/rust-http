//! A very simple HTTP server which responds with the plain text "Hello, World!" to every request.

extern mod extra;
extern mod http;

use std::rt::io::net::ip::{SocketAddr, Ipv4Addr};
use std::rt::io::Writer;
use extra::time;

use http::server::{Config, Server, ServerUtil, Request, ResponseWriter};
use http::headers::content_type::MediaType;

#[deriving(Clone)]
struct HelloWorldServer;

impl Server for HelloWorldServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8001 } }
    }

    fn handle_request(&self, _r: &Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.content_length = Some(14);
        w.headers.content_type = Some(MediaType {
            type_: ~"text",
            subtype: ~"plain",
            parameters: ~[(~"charset", ~"UTF-8")]
        });
        w.headers.server = Some(~"Example");

        w.write(bytes!("Hello, World!\n"));
    }
}

fn main() {
    HelloWorldServer.serve_forever();
}
