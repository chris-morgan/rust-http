//! SSL support provided by OpenSSL.

extern crate openssl;

use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::{IoResult, IoError, ConnectionAborted, OtherIoError};
use self::openssl::ssl::{SslStream, SslContext, Sslv23, Ssl};
use self::openssl::ssl::error::{SslError, StreamError, SslSessionClosed, OpenSslErrors};
use self::NetworkStream::{NormalStream, SslProtectedStream};
use connecter::Connecter;

/// A TCP stream, either plain text or SSL.
///
/// This build was made with **OpenSSL** providing SSL support.
pub enum NetworkStream {
    NormalStream(TcpStream),
    SslProtectedStream(SslStream<TcpStream>),
}

impl Connecter for NetworkStream {
    fn connect(addr: SocketAddr, host: &str, use_ssl: bool) -> IoResult<NetworkStream> {
        let stream = try!(TcpStream::connect(addr));
        if use_ssl {
            let context = try!(SslContext::new(Sslv23).map_err(lift_ssl_error));
            let ssl = try!(Ssl::new(&context).map_err(lift_ssl_error));
            try!(ssl.set_hostname(host).map_err(lift_ssl_error));
            let ssl_stream = try!(SslStream::new_from(ssl, stream).map_err(lift_ssl_error));
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

fn lift_ssl_error(ssl: SslError) -> IoError {
    match ssl {
        StreamError(err) => err,
        SslSessionClosed => IoError {
            kind: ConnectionAborted,
            desc: "SSL Connection Closed",
            detail: None
        },
        // Unfortunately throw this away. No way to support this
        // detail without a better Error abstraction.
        OpenSslErrors(errs) => IoError {
            kind: OtherIoError,
            desc: "Error in OpenSSL",
            detail: Some(format!("{}", errs))
        }
    }
}
