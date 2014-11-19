#![feature(macro_rules, slicing_syntax)]

use std::io::{File, Truncate, Write};
use std::os;

pub mod branchify;
pub mod status;
pub mod read_method;

fn main() {
    spawn(proc() {
        let output_dir = Path::new(os::getenv("OUT_DIR").unwrap());
        read_method::generate(output_dir).unwrap();
    });

    let output_dir = Path::new(os::getenv("OUT_DIR").unwrap());
    status::generate(output_dir).unwrap();
}

pub fn get_writer(mut output_dir: Path, filename: &str) -> Box<Writer + 'static> {
    output_dir.push(filename);
    match File::open_mode(&output_dir, Truncate, Write) {
        Ok(writer) => box writer as Box<Writer>,
        Err(e) => panic!("Unable to write file: {}", e.desc),
    }
}
