extern mod extra;

use std::cell::Cell;
use std::comm::SharedChan;
use std::task::{spawn_with, spawn_supervised};
use std::rt::io::{Listener, Acceptor, Writer};
use std::rt::io::net::ip::SocketAddr;
use std::rt::io::io_error;
use extra::time::precise_time_ns;

use std::rt::io::net::tcp::TcpListener;

use buffer::BufferedStream;

pub use self::request::{RequestBuffer, Request};
pub use self::response::ResponseWriter;

pub mod request;
pub mod response;

// TODO: when mozilla/rust#7661 is resolved, assuming also that specifying inheritance of kinds for
// the trait works:
// - Scrap ServerUtil (including any using it into the local scope)
// - Change "trait Server" to "trait Server: Send"
// - Shift the serve_forever method into Server
pub trait Server {
	fn handle_request(&self, request: &Request, response: &mut ResponseWriter) -> ();

	// XXX: this could also be implemented on the serve methods
	fn get_config(&self) -> Config;
}

/// A temporary trait to fix current deficiencies in Rust's default methods on traits.
/// You'll need to import `ServerUtil` to be able to call `serve_forever` on a Server.
pub trait ServerUtil {
    fn serve_forever(self);
}

impl<T: Send + Clone + Server> ServerUtil for T {
	/**
	 * Attempt to bind to the address and port and start serving forever.
	 *
	 * This will only return if the initial connection fails or something else blows up.
	 */
    fn serve_forever(self) {
        let config = self.get_config();
        debug!("About to bind to %?", config.bind_address);
        match TcpListener::bind(config.bind_address).listen() {
            None => {
                error!("bind or listen failed :-(");
                return;
            },
            Some(ref mut acceptor) => {
                debug!("listening");
                let (perf_po, perf_ch) = stream();
                let perf_ch = SharedChan::new(perf_ch);
                spawn_with(perf_po, perf_dumper);
                loop {
                    // OK, we're sort of shadowing an IoError here. Perhaps this should be done in a
                    // separate task so that it can safely fail...
                    let mut error = None;
                    let optstream = io_error::cond.trap(|e| {
                        error = Some(e);
                    }).inside(|| {
                        acceptor.accept()
                    });

                    let time_start = precise_time_ns();
                    if optstream.is_none() {
                        debug!("accept failed: %?", error);
                        // Question: is this the correct thing to do? We should probably be more
                        // intelligent, for there are some accept failures that are likely to be
                        // permanent, such that continuing would be a very bad idea, such as
                        // ENOBUFS/ENOMEM; and some where it should just be ignored, e.g.
                        // ECONNABORTED. TODO.
                        loop;
                    }
                    let stream = Cell::new(optstream.unwrap());
                    let child_perf_ch = perf_ch.clone();
                    let child_self = self.clone();
                    do spawn_supervised {
                        let mut time_start = time_start;
                        let mut stream = BufferedStream::new(stream.take(),
                                                             /* TcpStream.flush() fails! */ false);
                        debug!("accepted connection, got %?", stream);
                        loop {  // A keep-alive loop, condition at end
                            let time_spawned = precise_time_ns();
                            let (request, err_status) = Request::load(&mut stream);
                            let time_request_made = precise_time_ns();
                            let mut response = ~ResponseWriter::new(&mut stream, request);
                            let time_response_made = precise_time_ns();
                            match err_status {
                                Ok(()) => {
                                    child_self.handle_request(request, response);
                                    // Ensure that we actually do send a response:
                                    response.try_write_headers();
                                },
                                Err(status) => {
                                    // Uh oh, it's a response that I as a server cannot cope with.
                                    // No good user-agent should have caused this, so for the moment
                                    // at least I am content to send no body in the response.
                                    response.status = status;
                                    response.headers.content_length = Some(0);
                                    response.write_headers();
                                },
                            }
                            // Ensure the request is flushed, any Transfer-Encoding completed, etc.
                            response.finish_response();
                            let time_finished = precise_time_ns();
                            child_perf_ch.send((time_start, time_spawned, time_request_made, time_response_made, time_finished));

                            // Subsequent requests on this connection have no spawn time
                            time_start = time_finished;

                            if request.close_connection {
                                break;
                            }
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
	bind_address: SocketAddr,
}

/* Sorry, but Rust isn't ready for this yet; SimpleServer can't be made Clone just yet. (For
 * starters, ~fn() isn't cloneable.)

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
/// SimpleServer::new(Config { bind_address: socket_addr }, handler).serve_forever();
/// ~~~
///
/// But it's nicer this way with `do` blocks and closures:
///
/// ~~~ {.rust}
/// do serve_forever(socket_addr) |r, mut w| {
///     // Now you can handle the request here and write the wresponse.
/// }
/// ~~~
///
/// The signature of this method is liable to change if `Config` gets more members.
// Please, pretty please, don't correct the word "wresponse".
#[inline]
pub fn serve_forever(socket_addr: SocketAddr, handler: ~fn(&Request, &mut ResponseWriter)) {
    SimpleServer::new(Config { bind_address: socket_addr }, handler).serve_forever();
}

/// 0.0.0.0, port 80: publicly bound to the standard HTTP port.
/// Not recommended at present as this server is not hardened against the sort of traffic you may
/// encounter on the Internet and is vulnerable to various DoS attacks. Sit it behind a gateway.
static PUBLIC: SocketAddr = SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: 80 };
*/

static PERF_DUMP_FREQUENCY : u64 = 10_000;

/// Simple function to dump out perf stats every `PERF_DUMP_FREQUENCY` requests
fn perf_dumper(perf_po: Port<(u64, u64, u64, u64, u64)>) {
    // Total durations
    let mut td_spawn = 0u64;
    let mut td_request = 0u64;
    let mut td_response = 0u64;
    let mut td_handle = 0u64;
    let mut td_total = 0u64;
    let mut i = 0u64;
    loop {
        let data = perf_po.recv();
        let (start, spawned, request_made, response_made, finished) = data;
        td_spawn += spawned - start;
        td_request += request_made - spawned;
        td_response += response_made - request_made;
        td_handle += finished - response_made;
        td_total += finished - start;
        i += 1;
        if i % PERF_DUMP_FREQUENCY == 0 {
            println("");
            println(fmt!("%? requests made thus far. Current means:", i));
            println(fmt!("- Total:               100%%, %12?", td_total / i));
            println(fmt!("- Spawn:               %3?%%, %12?", 100 * td_spawn / td_total, td_spawn / i));
            println(fmt!("- Load request:        %3?%%, %12?", 100 * td_request / td_total, td_request / i));
            println(fmt!("- Initialise response: %3?%%, %12?", 100 * td_response / td_total, td_response / i));
            println(fmt!("- Handle:              %3?%%, %12?", 100 * td_handle / td_total, td_handle / i));
        }
    }
}
