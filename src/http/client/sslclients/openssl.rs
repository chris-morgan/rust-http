extern mod ssl(name = "rust-ssl");

use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use super::super::request::{NetworkStream, SslProtectedStream};

pub fn ssl_connect(addr: SocketAddr, host: ~str) -> Option<NetworkStream> {
    match TcpStream::connect(addr) {
        Some(stream) => Some(SslProtectedStream(ssl::SslStream::new(&ssl::SslContext::new(ssl::Sslv23), stream))),
        None => None
     }
}
