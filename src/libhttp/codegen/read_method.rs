use super::branchify::generate_branchified_method;
use super::get_writer;

pub fn generate(output_dir: &Path) {
    let writer = get_writer(output_dir, "read_method.rs");
    writer.write(bytes!("\
// This automatically generated file is included in request.rs.
{
    use method::{Connect, Delete, Get, Head, Options, Patch, Post, Put, Trace, ExtensionMethod};
    use server::request::MAX_METHOD_LEN;
    use rfc2616::{SP, is_token_item};

"));

    generate_branchified_method(
        writer,
        branchify!(case sensitive,
            "CONNECT" => Connect,
            "DELETE"  => Delete,
            "GET"     => Get,
            "HEAD"    => Head,
            "OPTIONS" => Options,
            "PATCH"   => Patch,
            "POST"    => Post,
            "PUT"     => Put,
            "TRACE"   => Trace
        ),
        1,
        "self.stream.read_byte()",
        "SP",
        "MAX_METHOD_LEN",
        "is_token_item(b)",
        "ExtensionMethod(%s)");
    writer.write(bytes!("}\n"));
}
