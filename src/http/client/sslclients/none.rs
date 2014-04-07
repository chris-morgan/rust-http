//! No SSL support (neither OpenSSL nor NSS were compiled in).

use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::{IoResult, IoError, InvalidInput};
use connecter::Connecter;

/// A TCP stream, plain text and with no SSL support.
///
/// This build was made *without* SSL support; if you attempt to make an SSL
/// connection you will receive an `IoError` of the `InvalidInput` kind.
///
/// (To build with SSL support, use ``--cfg openssl`` or ``--cfg nss``.)
pub enum NetworkStream {
    priv NormalStream(TcpStream),
}

impl Connecter for NetworkStream {
    fn connect(addr: SocketAddr, _host: &str, use_ssl: bool) -> IoResult<NetworkStream> {
        if use_ssl {
            Err(IoError {
                kind: InvalidInput,
                desc: "http crate was compiled without SSL support",
                detail: None,
            })
        } else {
            let stream = try!(TcpStream::connect(addr));
            Ok(NormalStream(stream))
        }
    }
}

impl Reader for NetworkStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        match *self {
            NormalStream(ref mut ns) => ns.read(buf),
        }
    }
}

impl Writer for NetworkStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match *self {
            NormalStream(ref mut ns) => ns.write(buf),
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        match *self {
            NormalStream(ref mut ns) => ns.flush(),
        }
    }
}
