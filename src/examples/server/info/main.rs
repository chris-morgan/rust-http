//! A not-quite-trivial HTTP server which responds to requests by showing the request and response
//! headers and any other information it has.

extern mod extra;
extern mod http;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;
use extra::time;

use http::server::{Config, Server, Request, ResponseWriter};
use http::headers::HeaderEnum;
use http::headers::content_type::MediaType;

#[deriving(Clone)]
struct InfoServer;

impl Server for InfoServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8001 } }
    }

    fn handle_request(&self, r: &Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.content_type = Some(MediaType {
            type_: ~"text",
            subtype: ~"html",
            parameters: ~[(~"charset", ~"UTF-8")]
        });
        w.headers.server = Some(~"Rust Thingummy/0.0-pre");
        w.write(bytes!("<!DOCTYPE html><title>Rust HTTP server</title>"));

        w.write(bytes!("<h1>Request</h1>"));
        let s = format!("<dl>
            <dt>Method</dt><dd>{}</dd>
            <dt>Host</dt><dd>{:?}</dd>
            <dt>Request URI</dt><dd>{:?}</dd>
            <dt>HTTP version</dt><dd>{:?}</dd>
            <dt>Close connection</dt><dd>{}</dd></dl>",
            r.method,
            r.headers.host,
            r.request_uri,
            r.version,
            r.close_connection);
        w.write(s.as_bytes());
        w.write(bytes!("<h2>Extension headers</h2>"));
        w.write(bytes!("<table><thead><tr><th>Name</th><th>Value</th></thead><tbody>"));
        for header in r.headers.iter() {
            let line = format!("<tr><td><code>{}</code></td><td><code>{}</code></td></tr>",
                               header.header_name(),
                               header.header_value());
            w.write(line.as_bytes());
        }
        w.write(bytes!("</tbody></table>"));
        w.write(bytes!("<h2>Body</h2><pre>"));
        w.write(r.body.as_bytes());
        w.write(bytes!("</pre>"));

        w.write(bytes!("<h1>Response</h1>"));
        let s = format!("<dl><dt>Status</dt><dd>{}</dd></dl>", w.status);
        w.write(s.as_bytes());
        w.write(bytes!("<h2>Headers</h2>"));
        w.write(bytes!("<table><thead><tr><th>Name</th><th>Value</th></thead><tbody>"));
        {
            let h = w.headers.clone();
            for header in h.iter() {
                let line = format!("<tr><td><code>{}</code></td><td><code>{}</code></td></tr>",
                                header.header_name(),
                                header.header_value());
                w.write(line.as_bytes());
            }
        }
        w.write(bytes!("</tbody></table>"));
    }
}

fn main() {
    InfoServer.serve_forever();
}
