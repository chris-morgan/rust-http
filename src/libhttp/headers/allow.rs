//! The Allow entity header, defined in RFC 2616, Section 14.7.

use method::Method;
use std::rt::io::Reader;
use headers::serialization_utils::comma_split_iter;

pub type Allow = ~[Method];

impl super::HeaderConvertible for Allow {
    fn from_stream<T: Reader>(reader: &mut super::HeaderValueByteIterator<T>) -> Option<Allow> {
        // TODO: Method::from_str_or_new needs to check is_token
        let mut output = ~[];
        let s = reader.collect_to_str();
        for name in comma_split_iter(s) {
            match Method::from_str_or_new(name) {
                Some(method) => output.push(method),
                None => return None,  // invalid method name (not a token)
            }
        }
        Some(output)
    }

    fn http_value(&self) -> ~str {
        let mut s = ~"";
        let mut first = true;
        for method in self.iter() {
            if !first {
                s.push_str(", ");
            } else {
                first = false;
            }
            s.push_str(method.to_str());
        }
        s
        // Why won't this work? self.iter().map(|method| method.to_str()).collect().connect(", ")
    }
}
