#[allow(default_methods)];  // For the benefit of Server. Hopefully it'll work.

extern mod extra;

use std::rt::io::Listener;
use std::rt::io::net::ip::{IpAddr, Ipv4};
use std::rt::io::io_error;

#[cfg(newrt)]
pub use std::rt::io::net::tcp::{TcpListener, TcpStream};

#[cfg(not(newrt))]
pub use TcpListener = super::adapter::ExtraNetTcpListener;
#[cfg(not(newrt))]
pub use TcpStream = super::adapter::ExtraNetTcpStream;

use super::request::{RequestBuffer, Request};
use super::response::ResponseWriter;

// TODO: when mozilla/rust#7661 is resolved, assuming also that specifying inheritance of kinds for
// the trait works:
// - Scrap ServerUtil (including any using it into the local scope)
// - Change "trait Server" to "trait Server: Send"
// - Shift the serve_forever method into Server
pub trait Server {
	pub fn handle_request(&self, request: &Request, response: &mut ResponseWriter) -> ();

	// XXX: this could also be implemented on the serve methods
	pub fn get_config(&self) -> Config;
}

/// A temporary trait to fix current deficiencies in Rust's default methods on traits.
/// You'll need to import `ServerUtil` to be able to call `serve_forever` on a Server.
pub trait ServerUtil {
    pub fn serve_forever(self);
}

impl<T: Send + Server> ServerUtil for T {
	/**
	 * Attempt to bind to the address and port and start serving forever.
	 *
	 * This will only return if the initial connection fails or something else blows up.
	 */
    pub fn serve_forever(self) {
        let config = self.get_config();
        debug!("About to bind to %?", config.bind_address);
        let mut optlistener = TcpListener::bind(config.bind_address);
        debug!("Bind attempt completed");
        match optlistener {
            None => {
                debug!("listen failed :-(");
                return;
            }
            Some(ref mut listener) => {
                debug!("listening");
                loop {
                    // OK, we're sort of shadowing an IoError here. Perhaps this should be done in a
                    // separate task so that it can safely fail...
                    let mut error = None;
                    let optstream = io_error::cond.trap(|e| {
                        error = Some(e);
                    }).in(|| {
                        listener.accept()
                    });

                    if optstream.is_none() {
                        debug!("accept failed: %?", error);
                        // Question: is this the correct thing to do? We should probably be more
                        // intelligent, for there are some accept failures that are likely to be
                        // permanent, such that continuing would be a very bad idea, such as
                        // ENOBUFS/ENOMEM; and some where it should just be ignored, e.g.
                        // ECONNABORTED. TODO.
                        loop;
                    }
                    let stream = ::std::cell::Cell::new(optstream.unwrap());
                    do spawn {
                        let mut stream = ~stream.take();
                        debug!("accepted connection, got %?", stream);
                        //RequestBuffer::new(stream);
                        let request = Request::get(~RequestBuffer::new(stream));
                        let mut response = ~ResponseWriter::new(*stream);
                        match request {
                            Ok(request) => {
                                self.handle_request(request, response);
                                // Sorry, only single-threaded at present:
                                // blocked on mozilla/rust#7661
                            },
                            Err(status) => {
                                response.status = status;
                                response.write_headers();
                            },
                        }
                    }
                }
            }
        }
    }
}

/// The necessary configuration for an HTTP server.
///
/// At present, only the IP address and port to bind to are needed, but it's possible that other
/// options may turn up later.
pub struct Config {
	bind_address: IpAddr,
}

/// A simple `Server`-implementing class, allowing code to be written in an imperative style.
pub struct SimpleServer {
    config: Config,
    handler: ~fn(&Request, &mut ResponseWriter),
}

impl SimpleServer {
    /// Create a new `SimpleServer` instance with the provided members.
    pub fn new(config: Config, handler: ~fn(&Request, &mut ResponseWriter)) -> SimpleServer {
        SimpleServer {
            config: config,
            handler: handler,
        }
    }
}

impl Server for SimpleServer {
    /// Handles a request by passing it on to the structure's handler function.
    #[inline]
	pub fn handle_request(&self, request: &Request, response: &mut ResponseWriter) {
        (self.handler)(request, response);
    }

    /// Returns the structure's known config.
    #[inline]
	pub fn get_config(&self) -> Config {
        self.config
    }
}

/// Create a simple server and immediately start serving forever.
///
/// This is equivalent to
///
/// ~~~ {.rust}
/// SimpleServer::new(Config { bind_address: ip_addr }, handler).serve_forever();
/// ~~~
///
/// But it's nicer this way with `do` blocks and closures:
///
/// ~~~ {.rust}
/// do serve_forever(ip_addr) |r, mut w| {
///     // Now you can handle the request here and write the wresponse.
/// }
/// ~~~
///
/// The signature of this method is liable to change if `Config` gets more members.
// Please, pretty please, don't correct the word "wresponse".
#[inline]
pub fn serve_forever(ip_addr: IpAddr, handler: ~fn(&Request, &mut ResponseWriter)) {
    SimpleServer::new(Config { bind_address: ip_addr }, handler).serve_forever();
}

/// 0.0.0.0, port 80: publicly bound to the standard HTTP port.
/// Not recommended at present as this server is not hardened against the sort of traffic you may
/// encounter on the Internet and is vulnerable to various DoS attacks. Sit it behind a gateway.
static PUBLIC: IpAddr = Ipv4(0, 0, 0, 0, 80);
