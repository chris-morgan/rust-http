//! The Accept-Ranges request header, defined in RFC 2616, Section 14.5.

use std::ascii::StrAsciiExt;
use std::rt::io::Reader;

#[deriving(Clone,Eq)]
// RFC 2616: range-unit = bytes-unit | other-range-unit
pub enum RangeUnit {
    Bytes,                   // bytes-unit       = "bytes"
    UnknownRangeUnit(~str),  // other-range-unit = token
}

#[deriving(Clone,Eq)]
// RFC 2616: acceptable-ranges = 1#range-unit | "none"
pub enum AcceptableRanges {
    RangeUnits(~[RangeUnit]),
    NoAcceptableRanges,
}

impl super::HeaderConvertible for AcceptRanges {
    fn from_stream<T: Reader>(reader: &mut super::HeaderValueByteIterator<T>)
            -> Option<AcceptRanges> {
        let mut range_units = ~[];
        loop {
            let token = reader.read_token().to_ascii_lower();
            match token.as_bytes() {
                "bytes" => range_units.push(Bytes),
                "none" if range_units.len() == 0 => return Some(NoAcceptableRanges),
                _ => range_units.push(OtherRangeUnit(token)),
            }
        }
        Some(RangeUnits(range_units))
    }

    fn to_stream<T: Writer>(&self, writer: &mut T) {
        match *self {
            NoAcceptableRanges => writer.write(bytes!("none")),
            RangeUnits(range_units) => for ru in range_units {
                match ru {
                    Bytes => writer.write(bytes!("bytes")),
                    UnknownRangeUnit(ref ru) => writer.write(ru.as_bytes()),
                }
            },
        }
    }

    fn http_value(&self) -> ~str {
        match *self {
            NoAcceptableRanges => ~"none",
            RangeUnits(range_units) => {
                let mut result = ~"";
                for ru in range_units {
                    match ru {
                        Bytes => result.push_str("bytes"),
                        UnknownRangeUnit(ref ru) => result.push_str(ru),
                    }
                }
                result
            },
        }
    }
}
