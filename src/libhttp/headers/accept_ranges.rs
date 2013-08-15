//! The Accept-Ranges request header, defined in RFC 2616, Section 14.4.

use std::ascii::to_ascii_lower;

pub enum RangeUnit {
    Unknown(~str),
    Bytes,
}
impl ToStr for RangeUnit {
    fn to_str(&self) -> ~str {
        match *self {
            Unknown(ref s) => s.clone(),
            Bytes => ~"bytes",
        }
    }
}
// More correct name would be AcceptableRanges, but I want to be consistent.
pub enum AcceptRanges {
    RangeUnit(RangeUnit),
    NoAcceptableRanges,  // Strictly, this is not a range-unit.
}
impl ToStr for AcceptRanges {
    fn to_str(&self) -> ~str {
        match *self {
            RangeUnit(ref ru) => ru.to_str(),
            NoAcceptableRanges => ~"none",
        }
    }
}

impl super::HeaderConvertible for AcceptRanges {
    fn from_stream<T: Reader>(reader: &mut HeaderValueByteIterator<T>) -> Option<AcceptRanges> {
        let s = reader.collect_to_str();
        match to_ascii_lower(s) {
            "none" => NoAcceptableRanges,
            "bytes" => RangeUnit(Bytes),
            value => RangeUnit(Unknown(value)),
        }
    }

    fn http_value(&self) -> ~str {
        self.to_str()
    }
}
