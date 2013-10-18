extern mod nss;

use std::io::net::ip::SocketAddr;
use super::super::request::{NetworkStream, SslProtectedStream};

pub fn ssl_connect(addr: SocketAddr, host: ~str) -> Option<NetworkStream> {
    let mut nss = nss::nss::NSS::new();
    nss.init(); //TODO: We should probably shutdown nss at some point...
    Some(SslProtectedStream(nss::ssl::SSLStream::connect(addr, host)))
}
