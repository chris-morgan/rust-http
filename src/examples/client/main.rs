#[crate_id = "client"];

extern crate http;
use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::os;
use std::str;
use std::io::{Reader, println};
use std::io::net::tcp::TcpStream;

fn main() {
    format!("{}", Get);
    let args = os::args();
    match args.len() {
        0 => unreachable!(),
        2 => make_and_print_request(args[1]),
        _ => {
            println!("Usage: {} URL", args[0]);
            return;
        },
    };
}

fn  bold(text: &str) -> ~str { "[1m"    + text + "[0m" }
fn green(text: &str) -> ~str { "[33;1m" + text + "[0m" }

fn make_and_print_request(url: ~str) {
    let request = RequestWriter::<TcpStream>::new(Get, from_str(url).expect("Invalid URL :-("))
                               .unwrap();

    println!("{}", green("Request"));
    println!("{}", green("======="));
    println!("");
    println!("{} {}",   bold("URL:"),            request.url.to_str());
    println!("{} {:?}", bold("Remote address:"), request.remote_addr);
    println!("{} {}",   bold("Method:"),         request.method);
    println!("{}",      bold("Headers:"));
    for header in request.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }

    println!("");
    println!("{}", green("Response"));
    println!("{}", green("========"));
    println!("");
    let mut response = match request.read_response() {
        Ok(response) => response,
        Err(_request) => fail!("This example can progress no further with no response :-("),
    };
    println!("{} {}", bold("Status:"), response.status);
    println!("{}", bold("Headers:"));
    for header in response.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }
    println!("{}", bold("Body:"));
    let body = response.read_to_end().unwrap();
    println(str::from_utf8(body).expect("Uh oh, response wasn't UTF-8"));
}
