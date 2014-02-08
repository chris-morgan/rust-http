//! The Transfer-Encoding request header, defined in RFC 2616, sections 14.41 and 3.6.
//!
//! Transfer-Encoding       = "Transfer-Encoding" ":" 1#transfer-coding

use std::ascii::StrAsciiExt;
use std::io::IoResult;
use headers::serialization_utils::{WriterUtil, push_parameters};

/// RFC 2616, section 3.6:
///
/// transfer-coding         = "chunked" | transfer-extension
/// transfer-extension      = token *( ";" parameter )
#[deriving(Clone,Eq)]
pub enum TransferCoding {
    Chunked,
    TransferExtension(~str, ~[(~str, ~str)]),
}

impl super::CommaListHeaderConvertible for TransferCoding {}

impl super::HeaderConvertible for TransferCoding {
    fn from_stream<R: Reader>(reader: &mut super::HeaderValueByteIterator<R>)
            -> Option<TransferCoding> {
        match reader.read_token() {
            Some(token) => {
                let token = token.to_ascii_lower();
                if token.as_slice() == "chunked" {
                    Some(Chunked)
                } else {
                    match reader.read_parameters() {
                        Some(parameters) => Some(TransferExtension(token, parameters)),
                        None => None,
                    }
                }
            }
            None => None,
        }
    }

    fn to_stream<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
        match *self {
            Chunked => writer.write(bytes!("chunked")),
            TransferExtension(ref token, ref parameters) => {
                if_ok!(writer.write_token(*token));
                writer.write_parameters(*parameters)
            }
        }
    }

    fn http_value(&self) -> ~str {
        match *self {
            Chunked => ~"chunked",
            TransferExtension(ref token, ref parameters) => {
                let out = token.to_owned();
                push_parameters(out, *parameters)
            }
        }
    }
}
