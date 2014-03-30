//! A sample HTTP server returning the same response as is returned by Apache httpd in its default
//! configuration. Potentially useful for a smidgeon of performance comparison, though naturally
//! Apache is doing a lot more than this does.

#![crate_id = "apache_fake"]

extern crate time;
extern crate http;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;

use http::server::{Config, Server, Request, ResponseWriter};
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
        //w.headers.last_modified = Some(~"Thu, 05 May 2011 11:46:42 GMT");
        w.headers.last_modified = Some(time::Tm {
            tm_sec: 42, // seconds after the minute ~[0-60]
            tm_min: 46, // minutes after the hour ~[0-59]
            tm_hour: 11, // hours after midnight ~[0-23]
            tm_mday: 5, // days of the month ~[1-31]
            tm_mon: 4, // months since January ~[0-11]
            tm_year: 111, // years since 1900
            tm_wday: 4, // days since Sunday ~[0-6]
            tm_yday: 0, // days since January 1 ~[0-365]
            tm_isdst: 0, // Daylight Savings Time flag
            tm_gmtoff: 0, // offset from UTC in seconds
            tm_zone: ~"GMT", // timezone abbreviation
            tm_nsec: 0, // nanoseconds
        });
        w.headers.etag = Some(headers::etag::EntityTag {
                                weak: false,
                                opaque_tag: ~"501b29-b1-4a285ed47404a" });
        w.headers.accept_ranges = Some(headers::accept_ranges::RangeUnits(
                                            vec!(headers::accept_ranges::Bytes)));
        w.headers.content_length = Some(177);
        w.headers.vary = Some(~"Accept-Encoding");
        w.headers.content_type = Some(headers::content_type::MediaType {
            type_: ~"text",
            subtype: ~"html",
            parameters: Vec::new()
        });
        w.headers.extensions.insert(~"X-Pad", ~"avoid browser bug");

        w.write(bytes!("\
            <html><body><h1>It works!</h1>\n\
            <p>This is the default web page for this server.</p>\n\
            <p>The web server software is running but no content has been added, yet.</p>\n\
            </body></html>\n")).unwrap();
    }
}

fn main() {
    ApacheFakeServer.serve_forever();
}
