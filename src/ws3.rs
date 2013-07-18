extern mod extra;
extern mod rusthttpserver;

use rusthttpserver::server::serve_forever;
use rusthttpserver::request::Request;
use rusthttpserver::response::ResponseWriter;
use std::rt::io::net::ip::Ipv4;
use std::rt::io::Writer;
use extra::time;

use rusthttpserver::server::{Config, Server, ServerUtil};
use rusthttpserver::rfc2616::format_http_time;

/// A copy of a request from Apache's default thingummy
struct ApacheFakeServer;
impl Server for ApacheFakeServer {
    fn get_config(&self) -> Config {
        Config { bind_address: Ipv4(127, 0, 0, 1, 8001) }
    }

    fn handle_request(&self, _r: &Request, w: &mut ResponseWriter) {
        w.headers.insert(~"Date", format_http_time(time::now_utc()));
        w.headers.insert(~"Server", ~"Apache/2.2.22 (Ubuntu)");
        w.headers.insert(~"Last-Modified", ~"Thu, 05 May 2011 11:46:42 GMT");
        w.headers.insert(~"ETag", ~"\"501b29-b1-4a285ed47404a\"");
        w.headers.insert(~"Accept-Ranges", ~"bytes");
        w.headers.insert(~"Content-Length", ~"177");
        w.headers.insert(~"Vary", ~"Accept-Encoding");
        w.headers.insert(~"Content-Type", ~"text/html");
        w.headers.insert(~"X-Pad", ~"avoid browser bug");

        w.write(bytes!("<html><body><h1>It works!</h1>
<p>This is the default web page for this server.</p>
<p>The web server software is running but no content has been added, yet.</p>
</body></html>\n"));
    }
}

fn main() {
    ApacheFakeServer.serve_forever();

    do serve_forever(Ipv4(127, 0, 0, 1, 8001)) |r, w| {
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
            let h = copy w.headers;
            for h.iter().advance |(k, v)| {
                let line = fmt!("<tr><td><code>%s</code></td><td><code>%s</code></td></tr>", *k, *v);
                w.write(line.as_bytes().to_owned());
            }
        }
        w.write(bytes!("</tbody></table>"));
    }

    do serve_forever(Ipv4(127, 0, 0, 1, 8001)) |_r, w| {
        w.headers.insert(~"Content-Length", ~"15");
        w.headers.insert(~"Content-Type", ~"text/plain; charset=UTF-8");
        w.headers.insert(~"Server", ~"Example");
        w.headers.insert(~"Date", ~"Wed, 17 Apr 2013 12:00:00 GMT");
        w.write(bytes!("Hello, World!"));
    }
}
