extern mod extra;

use librusthttpserver::server::{Server, HandlerFunc};
use librusthttpserver::response::ResponseWriter;
use std::rt::io::Writer;
use std::rt;

#[path = "librusthttpserver/mod.rs"]
mod librusthttpserver;

fn handler(mut r: ResponseWriter) {
	r.write(bytes!("Ooh! Wow!"));
}

fn main() {
	let hf = HandlerFunc(handler);
	let server = Server::new(extra::net::ip::v4::parse_addr("0.0.0.0"), 8001, &hf);
	println(fmt!("Serve finished: %?", server.serve_wait()));
}
