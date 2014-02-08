// The spelling "Connecter" is deliberate, by the way.

use std::io::IoResult;
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;

/// A trait for the concept of opening a stream connected to a IP socket address.
///
/// Why is this here? So that we can implement things which must make
/// connections in terms of *anything* that can make such a connection rather
/// than in terms of `TcpStream` only. This is handy for testing and for SSL.
pub trait Connecter {
    fn connect(addr: SocketAddr) -> IoResult<Self>;
}

impl Connecter for TcpStream {
    fn connect(addr: SocketAddr) -> IoResult<TcpStream> {
        TcpStream::connect(addr)
    }
}
