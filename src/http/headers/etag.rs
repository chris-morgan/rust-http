use headers::serialization_utils::{push_quoted_string, quoted_string, WriterUtil};
use std::io::IoResult;
use std::fmt;

#[deriving(Clone, Eq)]
pub struct EntityTag {
    weak: bool,
    opaque_tag: ~str,
}

pub fn weak_etag<S: Str>(opaque_tag: S) -> EntityTag {
    EntityTag {
        weak: true,
        opaque_tag: opaque_tag.into_owned(),
    }
}

pub fn strong_etag<S: Str>(opaque_tag: S) -> EntityTag {
    EntityTag {
        weak: false,
        opaque_tag: opaque_tag.into_owned(),
    }
}

impl fmt::Show for EntityTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.weak {
            f.buf.write(push_quoted_string(~"W/", self.opaque_tag).as_bytes())
        } else {
            f.buf.write(quoted_string(self.opaque_tag).as_bytes())
        }
    }
}

impl super::HeaderConvertible for EntityTag {
    fn from_stream<R: Reader>(reader: &mut super::HeaderValueByteIterator<R>) -> Option<EntityTag> {
        let weak = match reader.next() {
            Some(b) if b == 'W' as u8 || b == 'w' as u8 => {
                if reader.next() != Some('/' as u8) || reader.next() != Some('"' as u8) {
                    return None;
                }
                true
            },
            Some(b) if b == '"' as u8 => {
                false
            },
            _ => {
                return None;
            }
        };
        let opaque_tag = match reader.read_quoted_string(true) {
            Some(tag) => tag,
            None => return None,
        };
        reader.some_if_consumed(EntityTag {
            weak: weak,
            opaque_tag: opaque_tag,
        })
    }

    fn to_stream<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
        if self.weak {
            try!(writer.write(bytes!("W/")));
        }
        writer.write_quoted_string(self.opaque_tag)
    }

    fn http_value(&self) -> ~str {
        self.to_str()
    }
}

#[test]
fn test_etag() {
    use headers::test_utils::{assert_conversion_correct, assert_interpretation_correct,
                              assert_invalid};
    assert_conversion_correct("\"\"", strong_etag(""));
    assert_conversion_correct("\"fO0\"", strong_etag("fO0"));
    assert_conversion_correct("\"fO0 bar\"", strong_etag("fO0 bar"));
    assert_conversion_correct("\"fO0 \\\"bar\"", strong_etag("fO0 \"bar"));
    assert_conversion_correct("\"fO0 \\\"bar\\\"\"", strong_etag("fO0 \"bar\""));

    assert_conversion_correct("W/\"\"", weak_etag(""));
    assert_conversion_correct("W/\"fO0\"", weak_etag("fO0"));
    assert_conversion_correct("W/\"fO0 bar\"", weak_etag("fO0 bar"));
    assert_conversion_correct("W/\"fO0 \\\"bar\"", weak_etag("fO0 \"bar"));
    assert_conversion_correct("W/\"fO0 \\\"bar\\\"\"", weak_etag("fO0 \"bar\""));
    assert_interpretation_correct("w/\"fO0\"", weak_etag("fO0"));

    assert_invalid::<EntityTag>("");
    assert_invalid::<EntityTag>("fO0");
    assert_invalid::<EntityTag>("\"\\\"");
    assert_invalid::<EntityTag>("\"\"\"\"");
}
