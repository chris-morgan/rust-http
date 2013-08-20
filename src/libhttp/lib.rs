#[link(name = "http",
       vers = "0.1-pre",
       uuid = "d2ad8df0-547a-4ce1-99c6-a9da3b98fb3e",
       url = "")];

#[comment = "Rust HTTP server"];
#[license = "MIT/ASL2"];
#[crate_type = "lib"];

#[deny(non_camel_case_types)];
//#[deny(missing_doc)];

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
pub mod memstream;
