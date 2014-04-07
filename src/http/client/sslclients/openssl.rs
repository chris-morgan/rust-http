//! SSL support provided by OpenSSL.

extern crate openssl;

use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use self::openssl::ssl::{SslStream, SslContext, Sslv23};
use connecter::Connecter;

/// A TCP stream, either plain text or SSL.
///
/// This build was made with **OpenSSL** providing SSL support.
pub enum NetworkStream {
    priv NormalStream(TcpStream),
    priv SslProtectedStream(SslStream<TcpStream>),
}

impl Connecter for NetworkStream {
    fn connect(addr: SocketAddr, _host: &str, use_ssl: bool) -> IoResult<NetworkStream> {
        let stream = try!(TcpStream::connect(addr));
        if use_ssl {
            let ssl_stream = SslStream::new(&SslContext::new(Sslv23), stream);
            Ok(SslProtectedStream(ssl_stream))
        } else {
            Ok(NormalStream(stream))
        }
    }
}

impl Reader for NetworkStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        match *self {
            NormalStream(ref mut ns) => ns.read(buf),
            SslProtectedStream(ref mut ns) => ns.read(buf),
        }
    }
}

impl Writer for NetworkStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match *self {
            NormalStream(ref mut ns) => ns.write(buf),
            SslProtectedStream(ref mut ns) => ns.write(buf),
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        match *self {
            NormalStream(ref mut ns) => ns.flush(),
            SslProtectedStream(ref mut ns) => ns.flush(),
        }
    }
}
