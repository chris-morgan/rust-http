//! The Allow entity header, defined in RFC 2616, Section 14.7.

use method::Method;

pub type Allow = ~[Method];

impl ToStr for Allow {
    fn to_str(&self) -> ~str {
        self.iter().map(|method| method.to_str()).connect(", ")
    }
}

impl super::HeaderConvertible for Allow {
    fn from_stream<T: Reader>(reader: &mut HeaderValueByteIterator<T>) -> Option<Allow> {
        // TODO: Method::from_str_or_new needs to check is_token
        let output = ~[];
        for name in super::comma_split_iter(reader.collect_to_str()) {
            match Method::from_str_or_new(name) {
                Some(method) => output.push(method),
                None => return None,  // invalid method name (not a token)
            }
        }
        Some(output)
    }

    fn http_value(&self) -> ~str {
        self.to_str()
    }
}
