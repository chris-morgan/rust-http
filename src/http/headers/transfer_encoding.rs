//! The Transfer-Encoding request header, defined in RFC 2616, sections 14.41 and 3.6.
//!
//! Transfer-Encoding       = "Transfer-Encoding" ":" 1#transfer-coding

use std::ascii::AsciiExt;
use std::io::IoResult;
use headers::serialization_utils::{WriterUtil, push_parameters};

use self::TransferCoding::{Chunked, TransferExtension};

/// RFC 2616, section 3.6:
///
/// transfer-coding         = "chunked" | transfer-extension
/// transfer-extension      = token *( ";" parameter )
#[deriving(Clone, PartialEq, Eq)]
pub enum TransferCoding {
    Chunked,
    TransferExtension(String, Vec<(String, String)>),
}

impl super::CommaListHeaderConvertible for TransferCoding {}

impl super::HeaderConvertible for TransferCoding {
    fn from_stream<R: Reader>(reader: &mut super::HeaderValueByteIterator<R>)
            -> Option<TransferCoding> {
        match reader.read_token() {
            Some(token) => {
                // XXX is this actually the best way to do this?
                let token = token[].to_ascii_lower();
                if token[] == "chunked" {
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
            Chunked => writer.write(b"chunked"),
            TransferExtension(ref token, ref parameters) => {
                try!(writer.write_token(token));
                writer.write_parameters(parameters[])
            }
        }
    }

    fn http_value(&self) -> String {
        match *self {
            Chunked => String::from_str("chunked"),
            TransferExtension(ref token, ref parameters) => {
                push_parameters(token.clone(), parameters[])
            }
        }
    }
}
