//! The Accept-Ranges request header, defined in RFC 2616, Section 14.5.

use std::ascii::StrAsciiExt;
use std::io::{Reader, Writer};

#[deriving(Clone,Eq)]
// RFC 2616: range-unit = bytes-unit | other-range-unit
pub enum RangeUnit {
    Bytes,                 // bytes-unit       = "bytes"
    OtherRangeUnit(~str),  // other-range-unit = token
}

#[deriving(Clone,Eq)]
// RFC 2616: acceptable-ranges = 1#range-unit | "none"
pub enum AcceptableRanges {
    RangeUnits(~[RangeUnit]),
    NoAcceptableRanges,
}

impl super::HeaderConvertible for AcceptableRanges {
    fn from_stream<T: Reader>(reader: &mut super::HeaderValueByteIterator<T>)
            -> Option<AcceptableRanges> {
        let mut range_units = ~[];
        loop {
            match reader.read_token() {
                Some(token) => {
                    let token = token.to_ascii_lower();
                    match token.as_slice() {
                        "bytes" => range_units.push(Bytes),
                        "none" if range_units.len() == 0 => return Some(NoAcceptableRanges),
                        _ => range_units.push(OtherRangeUnit(token)),
                    }
                },
                None => break,
            }
        }
        Some(RangeUnits(range_units))
    }

    fn to_stream<T: Writer>(&self, writer: &mut T) {
        match *self {
            NoAcceptableRanges => writer.write(bytes!("none")),
            RangeUnits(ref range_units) => for ru in range_units.iter() {
                match ru {
                    &Bytes => writer.write(bytes!("bytes")),
                    &OtherRangeUnit(ref ru) => writer.write(ru.as_bytes()),
                }
            },
        }
    }

    fn http_value(&self) -> ~str {
        match *self {
            NoAcceptableRanges => ~"none",
            RangeUnits(ref range_units) => {
                let mut result = ~"";
                for ru in range_units.iter() {
                    match ru {
                        &Bytes => result.push_str("bytes"),
                        &OtherRangeUnit(ref ru) => result.push_str(*ru),
                    }
                }
                result
            },
        }
    }
}
