#![crate_name = "codegen"]

#![allow(unknown_features)]
#![feature(macro_rules, slicing_syntax)]

use std::io::{File, Truncate, Write};
use std::os;

pub mod branchify;
pub mod status;
pub mod read_method;

fn main() {
    let args = os::args();
    match args.len() {
        0 => {
            println!("usage: codegen [read_method|status].rs <output-dir>");
            os::set_exit_status(1);
        },
        3 => {
            let output_dir = Path::new(args[2][]);

            match args[1][] {
                "read_method.rs" => read_method::generate(&output_dir).unwrap(),
                "status.rs" => status::generate(&output_dir).unwrap(),
                s => {
                    println!("unknown thing-to-generate '{}'", s);
                    os::set_exit_status(1);
                }
            }
        },
        _ => {
            println!("usage: {} [read_method|status].rs <output-dir>", args[0]);
            os::set_exit_status(1);
        }
    }
}

pub fn get_writer(output_dir: &Path, filename: &str) -> Box<Writer + 'static> {
    let mut output_file = output_dir.clone();
    output_file.push(filename);
    match File::open_mode(&output_file, Truncate, Write) {
        Ok(writer) => box writer as Box<Writer>,
        Err(e) => panic!("Unable to write file: {}", e.desc),
    }
}
