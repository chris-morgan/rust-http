#[crate_id = "client"];

extern mod http;
use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::os;
use std::str;
use std::io::Reader;

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

fn make_and_print_request(url: ~str) {
    let request = RequestWriter::new(Get, from_str(url).expect("Invalid URL :-("));

    println("[33;1mRequest[0m");
    println("[33;1m=======[0m");
    println("");
    println!("[1mURL:[0m {}", request.url.to_str());
    println!("[1mRemote address:[0m {:?}", request.remote_addr);
    println!("[1mMethod:[0m {}", request.method);
    println("[1mHeaders:[0m");
    for header in request.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }

    println("");
    println("[33;1mResponse[0m");
    println("[33;1m========[0m");
    println("");
    let mut response = match request.read_response() {
        Ok(response) => response,
        Err(_request) => fail!("This example can progress no further with no response :-("),
    };
    println!("[1mStatus:[0m {}", response.status);
    println("[1mHeaders:[0m");
    for header in response.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }
    println("[1mBody:[0m");
    let body = response.read_to_end();
    println(str::from_utf8(body));
}
