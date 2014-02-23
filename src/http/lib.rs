#[crate_id = "http#0.1-pre"];

#[comment = "Rust HTTP server"];
#[license = "MIT/ASL2"];
#[crate_type = "dylib"];
#[crate_type = "rlib"];

#[doc(html_root_url = "http://www.rust-ci.org/chris-morgan/rust-http/doc/")];

#[deny(non_camel_case_types)];
//#[deny(missing_doc)];

#[feature(macro_rules)];
#[macro_escape];

extern crate extra;
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
#[path = "generated/status.rs"]
pub mod status;  // Getting an error? It's generated; use ``make`` or see the ``Makefile``

/// TODO: submit upstream
#[cfg(test)]
pub mod memstream;
