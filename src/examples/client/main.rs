extern mod http;
use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::str;
use std::rt::io::extensions::ReaderUtil;
use std::rt::io::net::ip::{SocketAddr, Ipv4Addr};

fn main() {
    let mut request = ~RequestWriter::new(Get, FromStr::from_str("http://localhost/example")
                                                .expect("Uh oh, that's *really* badly broken!"));
    // Temporary measure, as hostname lookup is not yet supported in std::rt::io.
    request.remote_addr = Some(SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8001 });
    let mut response = match request.read_response() {
        Ok(response) => response,
        Err(_request) => fail!("This example can progress no further with no response :-("),
    };
    println("Yay! Started to get the response.");
    println!("Status: {}", response.status);
    println("Headers:");
    for header in response.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }
    print("\n");
    println("Response:");
    println(str::from_utf8(response.read_to_end()));
}
