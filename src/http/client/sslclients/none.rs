use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use super::super::request::{NetworkStream, NormalStream};

pub fn ssl_connect(addr: SocketAddr, host: ~str) -> Option<NetworkStream> {
    match TcpStream::connect(addr) {
        Some(stream) => Some(NormalStream(stream)),
        None => None
     }
}
