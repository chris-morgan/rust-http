//! SSL support provided by NSS.

extern crate nss;

use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::IoResult;
use connecter::Connecter;

/// A TCP stream, either plain text or SSL.
///
/// This build was made with **NSS** providing SSL support.
pub enum NetworkStream {
    priv NormalStream(TcpStream),
    priv SslProtectedStream(nss::ssl::SSLStream),
}

impl Connecter for NetworkStream {
    fn connect(addr: SocketAddr, _host: &str, use_ssl: bool) -> IoResult<NetworkStream> {
        let stream = try!(TcpStream::connect(addr));
        if use_ssl {
            let mut nss = nss::nss::NSS::new();
            nss.init(); // TODO: we should probably shutdown NSS at some point.
            Ok(SslProtectedStream(nss::ssl::SSLStream::connect(addr, host)))
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
