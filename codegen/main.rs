#![allow(unstable)]
use std::io::{File, Truncate, Write};
use std::os;
use std::thread::Thread;

pub mod branchify;
pub mod status;
pub mod read_method;

fn main() {
    Thread::spawn(move || {
        let output_dir = Path::new(os::getenv("OUT_DIR").unwrap());
        read_method::generate(output_dir).unwrap();
    });

    let output_dir = Path::new(os::getenv("OUT_DIR").unwrap());
    status::generate(output_dir).unwrap();
}

pub fn get_writer(mut output_dir: Path, filename: &str) -> Box<Writer + 'static> {
    output_dir.push(filename);
    match File::open_mode(&output_dir, Truncate, Write) {
        Ok(writer) => Box::new(writer),
        Err(e) => panic!("Unable to write file: {}", e.desc),
    }
}
