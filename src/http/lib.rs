#![crate_name = "http"]

#![comment = "Rust HTTP server"]
#![license = "MIT/ASL2"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![doc(html_root_url = "http://www.rust-ci.org/chris-morgan/rust-http/doc/")]

#![deny(non_camel_case_types)]
//#[deny(missing_doc)];

#![allow(unknown_features)]
#![feature(slicing_syntax)]
#![feature(default_type_params)]
#![feature(macro_rules)]
#![feature(phase)]
#![feature(globs)]

#[phase(plugin, link)] extern crate log;
extern crate url;
extern crate time;
extern crate collections;

pub mod buffer;
pub mod client;
pub mod common;
pub mod connecter;
pub mod server;
pub mod method;
pub mod headers;
pub mod rfc2616;
include!(concat!(env!("OUT_DIR"), "/status.rs"))  // defines pub mod status

/// TODO: submit upstream
#[cfg(test)]
pub mod memstream;
