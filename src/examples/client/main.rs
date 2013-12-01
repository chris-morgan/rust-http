extern mod http;
use http::client::RequestWriter;
use http::client::ResponseReader;
use http::method::Get;
use http::headers::HeaderEnum;
use std::os;
use std::str;
use std::io::Reader;
use std::io::net::tcp::TcpStream;

// API Examples
use http::client::api::{get, RequestArgs};

fn main() {
    format!("{}", Get);
    let args = os::args();
    match args.len() {
        0 => unreachable!(),
        2 => get_example(args[1]),
        _ => {
            println!("Usage: {} URL", args[0]);
            return;
        },
    };
}

// NOTE(flaper87): Consider moving this to the
// request sender and print it if built in debug
// mode.
fn debug_request(request: &RequestWriter<TcpStream>) {
    println("[33;1mRequest[0m");
    println("[33;1m=======[0m");
    println("");
    println!("[1mURL:[0m {}", request.url.to_str());
    println!("[1mRemote address:[0m {:?}", request.remote_addr);
    println!("[1mMethod:[0m {}", request.method);
    println("[1mHeaders:[0m");
}

// NOTE(flaper87): Consider moving this to the
// request sender and print it if built in debug
// mode.
fn debug_response(response: &ResponseReader<TcpStream>) {
    println("");
    println("[33;1mResponse[0m");
    println("[33;1m========[0m");
    println("");
    println!("[1mStatus:[0m {}", response.status);
    println("[1mHeaders:[0m");
    for header in response.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }
}

fn get_example(url: ~str) {
    let params = ~[(~"test", ~"value")];
    let args = RequestArgs{params: Some(params), headers: None, data: None};
    let response = get(url, Some(args));

    let mut response = match response {
        Ok(response) => response,
        Err(_request) => fail!("This example can progress no further with no response :-("),
    };

    debug_response(&response);

    print("\n");
    println("Response:");
    let body = response.read_to_end();
    println(str::from_utf8_slice(body));
}

fn make_and_print_request(url: ~str) {
    let request = RequestWriter::new(Get, from_str(url).expect("Invalid URL :-("));

    debug_request(&request);
    for header in request.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }

    let mut response = match request.read_response() {
        Ok(response) => response,
        Err(_request) => fail!("This example can progress no further with no response :-("),
    };

    debug_response(&response);

    println("[1mBody:[0m");
    let body = response.read_to_end();
    println(str::from_utf8_slice(body));
}
