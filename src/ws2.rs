extern mod extra;

use librusthttpserver::server::{Server, Config, ServerUtil};
use librusthttpserver::request::Request;
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

	fn handle_request(&self, request: Request, mut r: ResponseWriter) {
        r.headers.insert(~"Content-Type", ~"text/html");
        r.headers.insert(~"Server", ~"Rust Thingummy/0.0-pre");
        r.write(bytes!("<!DOCTYPE html><title>Rust HTTP server</title>"));

		r.write(bytes!("<h1>Request</h1>"));
        let s = fmt!("<dl>
            <dt>Method</dt><dd>%s</dd>
            <dt>Path</dt><dd>%s</dd>
            <dt>HTTP version</dt><dd>%?</dd>
            <dt>Close connection</dt><dd>%?</dd></dl>",
            request.method.to_str(),
            request.path,
            request.version,
            request.close_connection);
        r.write(s.as_bytes().to_owned());
        r.write(bytes!("<h2>Headers</h2>"));
        r.write(bytes!("<table><thead><tr><th>Name</th><th>Value</th></thead><tbody>"));
        for request.headers.iter().advance |(k, v)| {
            let line = fmt!("<tr><td><code>%s</code></td><td><code>%s</code></td></tr>", *k, *v);
            r.write(line.as_bytes().to_owned());
        }
		r.write(bytes!("</tbody></table>"));
        r.write(bytes!("<h2>Body</h2><pre>"));
        r.write(request.body.as_bytes().to_owned());
        r.write(bytes!("</pre>"));

		r.write(bytes!("<h1>Response</h1>"));
        let s = fmt!("<dl><dt>Status</dt><dd>%s</dd></dl>", r.status.to_str());
		r.write(s.as_bytes().to_owned());
		r.write(bytes!("<h2>Headers</h2>"));
        r.write(bytes!("<table><thead><tr><th>Name</th><th>Value</th></thead><tbody>"));
        {
            let h = copy r.headers;
            for h.iter().advance |(k, v)| {
                let line = fmt!("<tr><td><code>%s</code></td><td><code>%s</code></td></tr>", *k, *v);
                r.write(line.as_bytes().to_owned());
            }
        }
		r.write(bytes!("</tbody></table>"));
	}
}

fn main() {
	println(fmt!("Serve finished: %?", MyServer.serve_forever()));
}
