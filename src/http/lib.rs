#![crate_name = "http"]

#![doc(html_root_url = "http://www.rust-ci.org/chris-morgan/rust-http/doc/")]

#![deny(non_camel_case_types)]
//#[deny(missing_doc)];

#![allow(unknown_features)]
#![feature(slicing_syntax)]
#![feature(old_orphan_check)]

#[macro_use] extern crate log;
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
include!(concat!(env!("OUT_DIR"), "/status.rs"));  // defines pub mod status

/// TODO: submit upstream
#[cfg(test)]
pub mod memstream;
