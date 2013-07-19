//! A not-quite-trivial HTTP server which responds to requests by showing the request and response
//! headers and any other information it has.

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
struct InfoServer;

impl Server for InfoServer {
    fn get_config(&self) -> Config {
        Config { bind_address: Ipv4(127, 0, 0, 1, 8001) }
    }

    fn handle_request(&self, r: &Request, w: &mut ResponseWriter) {
        w.headers.insert(~"Date", format_http_time(time::now_utc()));
        w.headers.insert(~"Content-Type", ~"text/html");
        w.headers.insert(~"Server", ~"Rust Thingummy/0.0-pre");
        w.write(bytes!("<!DOCTYPE html><title>Rust HTTP server</title>"));

        w.write(bytes!("<h1>Request</h1>"));
        let s = fmt!("<dl>
            <dt>Method</dt><dd>%s</dd>
            <dt>Path</dt><dd>%s</dd>
            <dt>HTTP version</dt><dd>%?</dd>
            <dt>Close connection</dt><dd>%?</dd></dl>",
            r.method.to_str(),
            r.path,
            r.version,
            r.close_connection);
        w.write(s.as_bytes().to_owned());
        w.write(bytes!("<h2>Headers</h2>"));
        w.write(bytes!("<table><thead><tr><th>Name</th><th>Value</th></thead><tbody>"));
        for r.headers.iter().advance |(k, v)| {
            let line = fmt!("<tr><td><code>%s</code></td><td><code>%s</code></td></tr>", *k, *v);
            w.write(line.as_bytes().to_owned());
        }
        w.write(bytes!("</tbody></table>"));
        w.write(bytes!("<h2>Body</h2><pre>"));
        w.write(r.body.as_bytes().to_owned());
        w.write(bytes!("</pre>"));

        w.write(bytes!("<h1>Response</h1>"));
        let s = fmt!("<dl><dt>Status</dt><dd>%s</dd></dl>", w.status.to_str());
        w.write(s.as_bytes().to_owned());
        w.write(bytes!("<h2>Headers</h2>"));
        w.write(bytes!("<table><thead><tr><th>Name</th><th>Value</th></thead><tbody>"));
        {
            let h = w.headers.clone();
            for h.iter().advance |(k, v)| {
                let line = fmt!("<tr><td><code>%s</code></td><td><code>%s</code></td></tr>", *k, *v);
                w.write(line.as_bytes().to_owned());
            }
        }
        w.write(bytes!("</tbody></table>"));
    }
}

fn main() {
    InfoServer.serve_forever();
}
