#[crate_id = "http#0.1-pre"];

#[comment = "Rust HTTP server"];
#[license = "MIT/ASL2"];
#[crate_type = "lib"];

#[deny(non_camel_case_types)];
//#[deny(missing_doc)];

#[feature(macro_rules)];
#[macro_escape];

extern mod extra;

pub mod buffer;
pub mod client;
pub mod common;
pub mod server;
pub mod method;
pub mod headers;
pub mod rfc2616;
#[path = "generated/status.rs"]
pub mod status;  // Getting an error? It's generated; use ``make`` or see the ``Makefile``

/// TODO: submit upstream
#[cfg(test)]
pub mod memstream;
