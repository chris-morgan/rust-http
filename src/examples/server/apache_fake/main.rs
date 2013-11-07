//! A sample HTTP server returning the same response as is returned by Apache httpd in its default
//! configuration. Potentially useful for a smidgeon of performance comparison, though naturally
//! Apache is doing a lot more than this does.

extern mod extra;
extern mod http;

use std::option::IntoOption;
use std::rt::io::net::ip::{SocketAddr, Ipv4Addr};
use std::rt::io::Writer;
use extra::time;

use http::server::{Config, Server, ServerUtil, Request, ResponseWriter};
use http::headers;

#[deriving(Clone)]
struct ApacheFakeServer;

impl Server for ApacheFakeServer {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8001 } }
    }

    fn handle_request(&self, _r: &Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.server = Some(~"Apache/2.2.22 (Ubuntu)");
        w.headers.last_modified =
            time::strptime("Thu, 05 May 2011 11:46:42 GMT",
                           "%a, %d %b %Y %H:%M:%S %Z").into_option();
        w.headers.etag = Some(headers::etag::EntityTag {
                                weak: false,
                                opaque_tag: ~"501b29-b1-4a285ed47404a" });
        w.headers.accept_ranges = Some(headers::accept_ranges::RangeUnits(
                                            ~[headers::accept_ranges::Bytes]));
        w.headers.content_length = Some(177);
        w.headers.vary = Some(~"Accept-Encoding");
        w.headers.content_type = Some(headers::content_type::MediaType {
            type_: ~"text",
            subtype: ~"html",
            parameters: ~[]
        });
        w.headers.extensions.insert(~"X-Pad", ~"avoid browser bug");

        w.write(bytes!("\
            <html><body><h1>It works!</h1>\n\
            <p>This is the default web page for this server.</p>\n\
            <p>The web server software is running but no content has been added, yet.</p>\n\
            </body></html>\n"));
    }
}

fn main() {
    ApacheFakeServer.serve_forever();
}
