extern crate extra;

use std::comm::Chan;
use std::io::{Listener, Acceptor};
use std::io::net::ip::SocketAddr;
use time::precise_time_ns;

use std::io::net::tcp::TcpListener;

use buffer::BufferedStream;

pub use self::request::{RequestBuffer, Request};
pub use self::response::ResponseWriter;

pub mod request;
pub mod response;

pub trait Server: Send + Clone {
	fn handle_request(&self, request: &Request, response: &mut ResponseWriter) -> ();

	// XXX: this could also be implemented on the serve methods
	fn get_config(&self) -> Config;

	/**
	 * Attempt to bind to the address and port and start serving forever.
	 *
	 * This will only return if the initial connection fails or something else blows up.
	 */
    fn serve_forever(self) {
        let config = self.get_config();
        debug!("About to bind to {:?}", config.bind_address);
        let mut acceptor = match TcpListener::bind(config.bind_address).listen() {
            Err(err) => {
                error!("bind or listen failed :-(: {}", err);
                return;
            },
            Ok(acceptor) => acceptor,
        };
        debug!("listening");
        let (perf_po, perf_ch) = Chan::new();
        spawn(proc() {
            perf_dumper(perf_po);
        });
        loop {
            let time_start = precise_time_ns();
            let stream = match acceptor.accept() {
                Err(error) => {
                    debug!("accept failed: {:?}", error);
                    // Question: is this the correct thing to do? We should probably be more
                    // intelligent, for there are some accept failures that are likely to be
                    // permanent, such that continuing would be a very bad idea, such as
                    // ENOBUFS/ENOMEM; and some where it should just be ignored, e.g.
                    // ECONNABORTED. TODO.
                    continue;
                },
                Ok(socket) => socket,
            };
            let child_perf_ch = perf_ch.clone();
            let child_self = self.clone();
            spawn(proc() {
                let mut time_start = time_start;
                let mut stream = BufferedStream::new(stream);
                debug!("accepted connection, got {:?}", stream);
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
                            match response.try_write_headers() {
                                Err(err) => {
                                    error!("Writing headers failed: {}", err);
                                    return;  // Presumably bad connection, so give up.
                                },
                                Ok(_) => (),
                            }
                        },
                        Err(status) => {
                            // Uh oh, it's a response that I as a server cannot cope with.
                            // No good user-agent should have caused this, so for the moment
                            // at least I am content to send no body in the response.
                            response.status = status;
                            response.headers.content_length = Some(0);
                            match response.write_headers() {
                                Err(err) => {
                                    error!("Writing headers failed: {}", err);
                                    return;  // Presumably bad connection, so give up.
                                },
                                Ok(_) => (),
                            }
                        },
                    }
                    // Ensure the request is flushed, any Transfer-Encoding completed, etc.
                    match response.finish_response() {
                        Err(err) => {
                            error!("finishing response failed: {}", err);
                            return;  // Presumably bad connection, so give up.
                        },
                        Ok(_) => (),
                    }
                    let time_finished = precise_time_ns();
                    child_perf_ch.send((time_start, time_spawned, time_request_made, time_response_made, time_finished));

                    // Subsequent requests on this connection have no spawn time
                    time_start = time_finished;

                    if request.close_connection {
                        break;
                    }
                }
            });
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
            println!("");
            println!("{} requests made thus far. Current means:", i);
            println!("- Total:               100%, {:12f}",
                     td_total as f64 / i as f64);
            println!("- Spawn:               {:3f}%, {:12f}",
                     100f64 * td_spawn as f64 / td_total as f64,
                     td_spawn as f64 / i as f64);
            println!("- Load request:        {:3f}%, {:12f}",
                     100f64 * td_request as f64 / td_total as f64,
                     td_request as f64 / i as f64);
            println!("- Initialise response: {:3f}%, {:12f}",
                     100f64 * td_response as f64 / td_total as f64,
                     td_response as f64 / i as f64);
            println!("- Handle:              {:3f}%, {:12f}",
                     100f64 * td_handle as f64 / td_total as f64,
                     td_handle as f64 / i as f64);
        }
    }
}
