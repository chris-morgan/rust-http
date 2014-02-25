//! The Content-Type entity header, defined in RFC 2616, Section 14.17.
use headers::serialization_utils::{push_parameters, WriterUtil};
use std::io::IoResult;
use std::fmt;

#[deriving(Clone, Eq)]
pub struct MediaType {
    type_: ~str,
    subtype: ~str,
    parameters: ~[(~str, ~str)],
}

pub fn MediaType(type_: ~str, subtype: ~str, parameters: ~[(~str, ~str)]) -> MediaType {
    MediaType {
        type_: type_,
        subtype: subtype,
        parameters: parameters,
    }
}

impl fmt::Show for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Idea:
        //let s = ~"";
        //s.push_token(self.type_);
        //s.push_char('/');
        //s.push_token(self.subtype);
        //s.push_parameters(self.parameters);
        //s
        let s = format!("{}/{}", self.type_, self.subtype);
        f.buf.write(push_parameters(s, self.parameters).as_bytes())
    }
}

impl super::HeaderConvertible for MediaType {
    fn from_stream<R: Reader>(reader: &mut super::HeaderValueByteIterator<R>) -> Option<MediaType> {
        let type_ = match reader.read_token() {
            Some(v) => v,
            None => return None,
        };
        if reader.next() != Some('/' as u8) {
            return None;
        }
        let subtype = match reader.read_token() {
            Some(v) => v,
            None => return None,
        };
        match reader.read_parameters() {
            // At the time of writing, ``Some(parameters) if reader.verify_consumed()`` was not
            // permitted: "cannot bind by-move into a pattern guard"
            Some(parameters) => {
                reader.some_if_consumed(MediaType {
                    type_: type_,
                    subtype: subtype,
                    parameters: parameters,
                })
            },
            None => None,
        }
    }

    fn to_stream<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
        try!(writer.write_token(self.type_));
        try!(writer.write(['/' as u8]));
        try!(writer.write_token(self.subtype));
        writer.write_parameters(self.parameters)
    }

    fn http_value(&self) -> ~str {
        self.to_str()
    }
}

#[test]
fn test_content_type() {
    use headers::test_utils::{assert_conversion_correct, assert_interpretation_correct,
                              assert_invalid};
    assert_conversion_correct("type/subtype", MediaType(~"type", ~"subtype", ~[]));
    assert_conversion_correct("type/subtype;key=value",
                              MediaType(~"type", ~"subtype", ~[(~"key", ~"value")]));
    assert_conversion_correct("type/subtype;key=value;q=0.1",
            MediaType(~"type", ~"subtype", ~[(~"key", ~"value"), (~"q", ~"0.1")]));
    assert_interpretation_correct("type/subtype ; key = value ; q = 0.1",
            MediaType(~"type", ~"subtype", ~[(~"key", ~"value"), (~"q", ~"0.1")]));

    assert_invalid::<MediaType>("");
    assert_invalid::<MediaType>("/");
    assert_invalid::<MediaType>("type/subtype,foo=bar");
    assert_invalid::<MediaType>("type /subtype");
    assert_invalid::<MediaType>("type/ subtype");
    assert_invalid::<MediaType>("type/subtype;foo=bar,foo=bar");
}
