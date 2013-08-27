//! The Transfer-Encoding request header, defined in RFC 2616, sections 14.41 and 3.6.
//!
//! Transfer-Encoding       = "Transfer-Encoding" ":" 1#transfer-coding

use std::ascii::StrAsciiExt;
use std::rt::io::Reader;

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
                        result.append(Chunked);
                    } else {
                        match reader.read_parameters() {
                            Some(parameters) => result.append(TransferExtension(token, parameters)),
                            None => return None,
                        }
                    }
                }
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
        for tc in self.iter() {
            match tc {
                Chunked => writer.write(bytes!("chunked")),
                TransferExtension(token, parameters) => {
                    writer.write_token(token);
                    writer.write_parameters(parameters);
                }
            }
            out.write(bytes!(", "));
        }
    }

    fn http_value(&self) -> ~str {
        let out = ~"";
        for tc in self.iter() {
            match tc {
                Chunked => out.push_str("chunked"),
                TransferExtension(token, parameters) => {
                    out.push_token(token);
                    out.push_parameters(parameters);
                }
            }
            out.push_str(", ");
        }
        out
    }
}
