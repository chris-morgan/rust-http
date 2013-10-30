#[feature(macro_rules)];

use std::rt::io::{Writer, CreateOrTruncate};
use std::rt::io::file::FileInfo;
use std::os;

pub mod branchify;
pub mod status;
pub mod read_method;

fn main() {
    let args = os::args();
    match args.len() {
        0 => {
            println("usage: codegen [read_method|status].rs");
            os::set_exit_status(1); 
        },
        3 => {
            let output_dir = from_str(args[2]).unwrap();
            // TODO: maybe not 0777?
            os::make_dir(&output_dir, 0b111_111_111);

            match args[1] {
                ~"read_method.rs" => read_method::generate(&output_dir),
                ~"status.rs" => status::generate(&output_dir),
                s => {
                    println!("unknown thing-to-generate '{}'", s);
                    os::set_exit_status(1);
                }
            }
        },
        _ => {
            println!("usage: {} [read_method|status].rs", args[0]);
            os::set_exit_status(1);
        }
    }
}

pub fn get_writer(output_dir: &Path, filename: &str) -> ~Writer {
    let mut output_file = output_dir.clone();
    output_file.push(filename);
    match output_file.open_writer(CreateOrTruncate) {
        Some(writer) => ~writer as ~Writer,
        None => fail!("Unable to write file"),
    }
}
