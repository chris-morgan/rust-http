extern mod extra;
extern mod rusthttpserver;

use rusthttpserver::server::{Server, Config, ServerUtil};
use rusthttpserver::request::Request;
use rusthttpserver::response::ResponseWriter;
use std::rt::io::net::ip::Ipv4;
use std::rt::io::Writer;

struct MyServer;

impl Server for MyServer {
	pub fn get_config(&self) -> Config {
		Config {
			bind_address: Ipv4(127, 0, 0, 1, 8001),
		}
	}

    fn handle_request(&self, request: Request, mut r: ResponseWriter) {
        //r.headers.insert(~"Content-Length", ~"15");
        //r.headers.insert(~"Content-Type", ~"text/plain; charset=UTF-8");
        //r.headers.insert(~"Server", ~"Example");
        //r.headers.insert(~"Date", ~"Wed, 17 Apr 2013 12:00:00 GMT");
        //r.write(bytes!("Hello, World!"));


        r.headers.insert(~"Date", ~"Tue, 16 Jul 2013 03:43:34 GMT");
        r.headers.insert(~"Server", ~"Apache/2.2.22 (Ubuntu)");
        r.headers.insert(~"Last-Modified", ~"Thu, 05 May 2011 11:46:42 GMT");
        r.headers.insert(~"ETag", ~"\"501b29-b1-4a285ed47404a\"");
        r.headers.insert(~"Accept-Ranges", ~"bytes");
        r.headers.insert(~"Content-Length", ~"177");
        r.headers.insert(~"Vary", ~"Accept-Encoding");
        r.headers.insert(~"Content-Type", ~"text/html");
        r.headers.insert(~"X-Pad", ~"avoid browser bug");

        r.write(bytes!("<html><body><h1>It works!</h1>
<p>This is the default web page for this server.</p>
<p>The web server software is running but no content has been added, yet.</p>
</body></html>\n"));
    }

	/*fn handle_request(&self, request: Request, mut r: ResponseWriter) {
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
	}*/
}

fn main() {
	println(fmt!("Serve finished: %?", MyServer.serve_forever()));
}
