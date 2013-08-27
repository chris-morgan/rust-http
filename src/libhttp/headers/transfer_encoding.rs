//! The Transfer-Encoding request header, defined in RFC 2616, sections 14.41 and 3.6.
//!
//! Transfer-Encoding       = "Transfer-Encoding" ":" 1#transfer-coding

use std::ascii::StrAsciiExt;
use std::rt::io::{Reader, Writer};
use headers::serialization_utils::{WriterUtil, push_parameters};
use headers::{CommaConsumed, EndOfValue, ErrCommaNotFound};

/// RFC 2616, section 3.6:
///
/// transfer-coding         = "chunked" | transfer-extension
/// transfer-extension      = token *( ";" parameter )
#[deriving(Clone,Eq)]
pub enum TransferCoding {
    Chunked,
    TransferExtension(~str, ~[(~str, ~str)]),
}

impl super::HeaderConvertible for ~[TransferCoding] {
    fn from_stream<T: Reader>(reader: &mut super::HeaderValueByteIterator<T>)
            -> Option<~[TransferCoding]> {
        let mut result = ~[];
        loop {
            match reader.read_token() {
                Some(token) => {
                    let token = token.to_ascii_lower();
                    if token.as_slice() == "chunked" {
                        result.push(Chunked);
                    } else {
                        match reader.read_parameters() {
                            Some(parameters) => result.push(TransferExtension(token, parameters)),
                            None => return None,
                        }
                    }
                }
                None => return None,
            }
            match reader.consume_comma_lws() {
                CommaConsumed => loop,
                EndOfValue => break,
                ErrCommaNotFound => return None,
            }
        }
        Some(result)
    }

    fn to_stream<T: Writer>(&self, writer: &mut T) {
        for (i, tc) in self.iter().enumerate() {
            if i != 0 {
                writer.write(bytes!(", "));
            }
            match *tc {
                Chunked => writer.write(bytes!("chunked")),
                TransferExtension(ref token, ref parameters) => {
                    writer.write_token(*token);
                    writer.write_parameters(*parameters);
                }
            }
        }
    }

    fn http_value(&self) -> ~str {
        let mut out = ~"";
        for (i, tc) in self.iter().enumerate() {
            if i != 0 {
                out.push_str(", ");
            }
            match *tc {
                Chunked => out.push_str("chunked"),
                TransferExtension(ref token, ref parameters) => {
                    out.push_str(*token);
                    out = push_parameters(out, *parameters);
                }
            }
        }
        out
    }
}
