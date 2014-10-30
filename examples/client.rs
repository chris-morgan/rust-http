#![crate_name = "client"]

#![allow(unknown_features)]
#![feature(slicing_syntax)]

extern crate http;
extern crate url;
use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::os;
use std::str;
use std::io::println;
use url::Url;

fn main() {
    format!("{}", Get);
    let args = os::args();
    match args.len() {
        0 => unreachable!(),
        2 => make_and_print_request(args[1][]),
        _ => {
            println!("Usage: {} URL", args[0]);
            return;
        },
    };
}

fn make_and_print_request(url: &str) {
    let url = Url::parse(url).ok().expect("Invalid URL :-(");
    let request: RequestWriter = RequestWriter::new(Get, url).unwrap();

    println!("[33;1mRequest[0m");
    println!("[33;1m=======[0m");
    println!("");
    println!("[1mURL:[0m {}", request.url);
    println!("[1mRemote address:[0m {}", request.remote_addr);
    println!("[1mMethod:[0m {}", request.method);
    println!("[1mHeaders:[0m");
    for header in request.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }

    println!("");
    println!("[33;1mResponse[0m");
    println!("[33;1m========[0m");
    println!("");
    let mut response = match request.read_response() {
        Ok(response) => response,
        Err(_request) => panic!("This example can progress no further with no response :-("),
    };
    println!("[1mStatus:[0m {}", response.status);
    println!("[1mHeaders:[0m");
    for header in response.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }
    println!("[1mBody:[0m");
    let body = match response.read_to_end() {
        Ok(body) => body,
        Err(err) => panic!("Reading response failed: {}", err),
    };
    println(str::from_utf8(body[]).expect("Uh oh, response wasn't UTF-8"));
}
