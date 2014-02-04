#[crate_id = "codegen"];

#[feature(macro_rules)];

use std::io::{File, Truncate, Write, fs};
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
            let output_dir = Path::new(args[2].as_slice());
            // TODO: maybe not 0777?
            if !output_dir.exists() {
                fs::mkdir(&output_dir, 0b111_111_111).unwrap();
            }

            match args[1] {
                ~"read_method.rs" => read_method::generate(&output_dir).unwrap(),
                ~"status.rs" => status::generate(&output_dir).unwrap(),
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
    match File::open_mode(&output_file, Truncate, Write) {
        Ok(writer) => ~writer as ~Writer,
        Err(e) => fail!("Unable to write file: {}", e.desc),
    }
}
