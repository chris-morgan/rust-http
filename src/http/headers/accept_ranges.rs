//! The Accept-Ranges request header, defined in RFC 2616, Section 14.5.

use std::io::IoResult;
use std::ascii::StrAsciiExt;

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

// Merely here because of mozilla/rust#11641
static BYTES: &'static [u8] = bytes!("bytes");

impl super::HeaderConvertible for AcceptableRanges {
    fn from_stream<R: Reader>(reader: &mut super::HeaderValueByteIterator<R>)
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

    fn to_stream<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
        match *self {
            NoAcceptableRanges => writer.write(bytes!("none")),
            RangeUnits(ref range_units) => {
                for ru in range_units.iter() {
                    try!(writer.write(match *ru {
                        Bytes => BYTES,
                        OtherRangeUnit(ref ru) => ru.as_bytes(),
                    }));
                }
                Ok(())
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
