extern mod extra;

use librusthttpserver::server::{Server, Config};
use librusthttpserver::response::ResponseWriter;
use std::rt::io::net::ip::Ipv4;
use std::rt::io::Writer;

#[path = "librusthttpserver/mod.rs"]
mod librusthttpserver;

struct MyServer;

impl Server for MyServer {
	pub fn get_config(&self) -> Config {
		Config {
			bind_address: Ipv4(127, 0, 0, 1, 8001),
		}
	}

	fn handle_request(&self, mut r: ResponseWriter) {
		r.write(bytes!("Ooh! Wow!"));
	}
}

fn main() {
    debug!("main");
	println(fmt!("Serve finished: %?", MyServer.serve_forever()));
}
