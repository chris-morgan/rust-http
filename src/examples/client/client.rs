extern mod http;
use http::client::RequestWriter;
use http::method::{Get, Post};
use http::headers::HeaderEnum;
use std::str;
use std::rt::io::extensions::ReaderUtil;

fn get_request() {
    let request = ~RequestWriter::new(Get, FromStr::from_str("http://httpbin.org/get")
                                                .expect("Uh oh, that's *really* badly broken!"));

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


fn post_request() {
    let mut request = ~RequestWriter::new(Post, FromStr::from_str("http://httpbin.org/post")
                                                .expect("Uh oh, that's *really* badly broken!"));

    request.send(bytes!("Post It!"));

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

fn main() {

    get_request();

    post_request();
}
