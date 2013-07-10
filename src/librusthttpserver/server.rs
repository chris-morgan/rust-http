#[allow(default_methods)];  // For the benefit of Server. Hopefully it'll work.

extern mod extra;

use std::option::Option;
use std::rt::io::{Listener};
use std::rt::io::net::tcp::{TcpListener, TcpStream};
use std::rt::io::net::ip::IpAddr;
use std::rt::io::{io_error, IoError};

use super::response::ResponseWriter;
//use self::adapter::WriterRtWriterAdapter;

mod adapter;

// TODO: specifying inheritance like this doesn't work with kinds at present :-(
pub trait Server: Send {

	pub fn handle_request(&self, r: ResponseWriter) -> ();

	// XXX: this could also be implemented on the serve methods
	pub fn get_config(&self) -> Config;

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
                    let mut error : Option<IoError> = None;
                    let mut optstream : Option<TcpStream> = None;
                    io_error::cond.trap(|e: IoError| {
                        error = Some(e);
                    }).in(|| {
                        optstream = listener.accept();
                    });
                    match optstream {
                        None => {
                            debug!("accept failed: %?", error);
                            // Question: is this the correct thing to do? We should probably be more
                            // intelligent, for there are some accept failures that are likely to be
                            // permanent, such that continuing would be a very bad idea, such as
                            // ENOBUFS/ENOMEM; and some where it should just be ignored, e.g.
                            // ECONNABORTED. TODO.
                            loop;
                        },
                        Some(stream) => {
                            debug!("accepted connection, got %?", stream);
                            //let stream = Cell::new(stream.get());
                            //do spawn {
                            //let stream = stream.take();
                            self.handle_request(ResponseWriter::new(stream));
                            // Sorry, only single-threaded at present: blocked on mozilla/rust#7661
                            //}
                        }
                    }
                }
            }
        }
    }
}

pub struct Config {
	bind_address: IpAddr,
}

/*
pub struct SimpleServer {
	config: Config,
}

impl Server for SimpleServer {
	pub fn get_config(&self) -> Config {
		self.config
	}
}

impl SimpleServer {
	/// Create a `Server` bound to all IPv4 addresses on port 80
	#[inline]
	pub fn new_public(callback: &'self T) -> Server<'self, T> {
		Server {
			bind_address: ip::v4::parse_addr("0.0.0.0"),
			port: 80,
			backlog: 100,
			callback: callback,
		}
	}

	/// Create a `Server` with the specified config parameters
	#[inline]
	pub fn new(bind_address: ip::IpAddr, port: uint, callback: &'self T) -> Server<'self, T> {
		SimpleServer {
			config: Config {
				bind_address: bind_address,
				port: port,
				backlog: 100,
				callback: callback,
			}
		}
	}
}
*/
