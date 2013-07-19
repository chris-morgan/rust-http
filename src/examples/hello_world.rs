//! A very simple HTTP server which responds with the plain text "Hello, World!" to every request.

extern mod extra;
extern mod rusthttpserver;

use rusthttpserver::request::Request;
use rusthttpserver::response::ResponseWriter;
use std::rt::io::net::ip::Ipv4;
use std::rt::io::Writer;
use extra::time;

use rusthttpserver::server::{Config, Server, ServerUtil};
use rusthttpserver::rfc2616::format_http_time;

#[deriving(Clone)]
struct HelloWorldServer;

impl Server for HelloWorldServer {
    fn get_config(&self) -> Config {
        Config { bind_address: Ipv4(127, 0, 0, 1, 8001) }
    }

    fn handle_request(&self, _r: &Request, w: &mut ResponseWriter) {
        w.headers.insert(~"Date", format_http_time(time::now_utc()));
        w.headers.insert(~"Content-Length", ~"15");
        w.headers.insert(~"Content-Type", ~"text/plain; charset=UTF-8");
        w.headers.insert(~"Server", ~"Example");

        w.write(bytes!("Hello, World!"));
    }
}

fn main() {
    HelloWorldServer.serve_forever();
}
