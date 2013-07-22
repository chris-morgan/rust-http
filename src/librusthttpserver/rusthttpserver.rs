#[link(name = "rusthttpserver",
       vers = "0.1-pre",
       uuid = "d2ad8df0-547a-4ce1-99c6-a9da3b98fb3e",
       url = "")];

#[comment = "Rust HTTP server"];
#[license = "MIT/ASL2"];
#[crate_type = "lib"];

#[deny(non_camel_case_types)];
//#[deny(missing_doc)];

extern mod extra;

pub mod server;
pub mod method;
pub mod status;
pub mod headers;
pub mod response;
pub mod request;
pub mod rfc2616;
