use std::io::{file_writer, Create, Truncate};
use std::os;

pub mod branchify;
pub mod status;
pub mod read_method;

fn main() {
    // TODO: maybe not 0777?
    os::make_dir(&output_dir(), 0b111_111_111);

    let args = os::args();
    match args.len() {
        0 => {
            println("usage: codegen [read_method|status].rs");
            os::set_exit_status(1);
        },
        2 => match args[1] {
            ~"read_method.rs" => read_method::generate(),
            ~"status.rs" => status::generate(),
            s => {
                printfln!("unknown thing-to-generate '%s'", s);
                os::set_exit_status(1);
            }
        },
        _ => {
            printfln!("usage: %s [read_method|status].rs", args[0]);
            os::set_exit_status(1);
        }
    }
}

fn output_dir() -> Path {
    os::self_exe_path().unwrap().with_filename("generated")  // i.e. ../generated/
}

pub fn get_writer(filename: &str) -> @Writer {
    match file_writer(&output_dir().push(filename), [Create, Truncate]) {
        Ok(writer) => writer,
        Err(msg) => fail!("Unable to write file: %s", msg),
    }
}
