//! A sample HTTP server returning the same response as is returned by Apache httpd in its default
//! configuration. Potentially useful for a smidgeon of performance comparison, though naturally
//! Apache is doing a lot more than this does.

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
}
