#[allow(default_methods)];  // For the benefit of Server. Hopefully it'll work.

extern mod extra;

use std::comm::SharedChan;
use std::result::{Result,Ok,Err};
use std::task;
use std::option::Option;
use extra::net;
use extra::net::tcp::{TcpNewConnection, TcpListenErrData, TcpErrData, TcpSocketBuf, socket_buf};
//use extra::net::tcp;
use extra::net::ip;
use extra::uv;
use std::rt;

use super::response::ResponseWriter;
use self::adapter::WriterRtWriterAdapter;

mod adapter;

pub trait Handler {
	pub fn handle_request(&self, r: ResponseWriter) -> ();
}

pub struct HandlerFunc<'self> {
	func: &'self fn(ResponseWriter),
}

pub fn HandlerFunc<'a>(func: &'a fn(ResponseWriter)) -> HandlerFunc<'a> {
	HandlerFunc { func: func }
}

/*impl<'self, T: rt::io::Writer> HandlerFunc<'self, T> {
	pub fn new(func: &'self fn(ResponseWriter)) -> HandlerFunc<'self, T> {
		HandlerFunc { func: func }
	}
}*/

impl<'self> Handler for HandlerFunc<'self> {
	#[inline]
	fn handle_request(&self, r: ResponseWriter) {
		(self.func)(r);
	}
}

type ConnectMsg = (TcpNewConnection, SharedChan<Option<TcpErrData>>);

enum ServeResultOK {
	Finished,
	KillChan(SharedChan<Option<TcpErrData>>),
}

type ServeResult = Result<ServeResultOK, TcpListenErrData>;

pub struct Server<'self, T> {
	bind_address: ip::IpAddr,
	port: uint,
	backlog: uint,
	callback: &'self T,
}

impl<'self, T: Handler> Server<'self, T> {

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
		Server {
			bind_address: bind_address,
			port: port,
			backlog: 100,
			callback: callback,
		}
	}

	/**
	 * Attempt to bind to the address and port and start serving forever.
	 *
	 * This will only return if the connection fails or the server is stopped from inside itself.
	 * (I haven't yet determined whether that will be possible or not.)
	 */
	pub fn try_serve(&self) -> Port<ServeResult> {
		// TODO: return a result of the kill_ch
		let (result_port, result_chan) = stream::<ServeResult>();
		// Alternative: let local_self = copy self;
		let bind_address = copy self.bind_address;
		let port = copy self.port;
		let backlog = copy self.backlog;
		let callback = self.callback;
		//let (result_port, result_chan): (Port<ServeResult>, Chan<ServeResult>) = stream();
		do spawn {
			match net::tcp::listen(
				bind_address,
				port,
				backlog,
				&uv::global_loop::get(),
				|kill_ch| {
					debug!("HTTP server connection established");
					// Provide external control of the server: give the caller the kill channel
					result_chan.send(Ok(KillChan(kill_ch)));
				},
				|new_conn, kill_ch| {
					debug!("Connection received");

					let (cont_po, cont_ch) = stream::<Option<TcpErrData>>();
					do task::spawn {
						let accept_result = net::tcp::accept(new_conn);
						match accept_result {
							Err(accept_error) => {
								debug!(fmt!("Accept error: %?", accept_error));
								cont_ch.send(Some(accept_error));
							},
							Ok(socket) => {
								cont_ch.send(None);
								// do work here

								// TODO: request
								let writer = ~WriterRtWriterAdapter(socket_buf(socket))
									as ~rt::io::Writer;
								let r = ResponseWriter::new(writer);
								callback.handle_request(r);
								// TODO: probably need to do something about Connection: close
							}
						}
					};
					match cont_po.recv() {
						// Shut down listen(). TODO: this is VERY DANGEROUS
						// thing to be doing. We should be more intelligent
						// about it. Do we *really* want to kill the server for
						// a measly ECONNABORTED?
						Some(err_data) => (), //kill_ch.send(Some(err_data)),
						// Wait for next connection.
						None => ()
					}
				}
			) {
				Err(tcp_listen_err_data) => {
					result_chan.send(Err(tcp_listen_err_data));
				}
				Ok(*) => {
					// Clean shutdown. result_chan will have already been sent kill_chan
					result_chan.send(Ok(Finished));
				}
			}
		};
		result_port
	}

	/**
	 * Bind to the address and port and start serving and then immediately return.
	 *
	 * This will fail if binding to the address or listening fails.
	 */
	pub fn serve(&self) {
		match self.try_serve().recv() {
			Ok(*) => {}
			Err(err) => {
				fail!(fmt!("Failed listen: %?", err));
			}
		}
	}

	/**
	 * Bind to the address and port and start serving until the server is stopped internally.
	 *
	 * This will fail if binding to the address or listening fails.
	 */
	pub fn serve_wait(&self) {
		let port = self.try_serve();
		loop {
			match port.recv() {
				Ok(val) => {
					match val {
						Finished => return,
						KillChan(*) => (),
					}
				}
				Err(err) => {
					fail!(fmt!("Failed listen: %?", err));
				}
			}
		}
	}
}
