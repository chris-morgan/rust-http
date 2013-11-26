/*!
 * Poorly categorised functions for reading things used in multiple places.
 *
 * (That is, typically, more than one of request-or-response reading-or-writing.)
 *
 * TODO: refactor all this to store things in more usefully categorised places.
 */
use std::num::{Zero, cast};
use std::io::Reader;
#[cfg(test)]
use std::io::mem::MemReader;

/**
 * Read a positive decimal integer from the given reader.
 *
 * # Arguments
 *
 * - `reader` - the reader to read the decimal digits from (and whatever byte comes next)
 * - `expected_end` - this function is called with the byte that follows the last decimal digit
 *   and should return `true` if that byte is what is expected to follow the number.
 *
 * # Return value
 *
 * - `None`, if the number overflows;
 * - `None`, if there is a leading zero;
 * - `None`, if all the digits are read and the `expected_end` function is false
 *
 * Should everything work as designed (i.e. none of these conditions occur) a `Some` is returned.
 */
pub fn read_decimal<R: Reader, N: Unsigned + NumCast + Ord>
                   (reader: &mut R, expected_end: |u8| -> bool)
                   -> Option<N> {
    // Here and in `read_hexadecimal` there is the possibility of infinite sequence of zeroes. The
    // spec allows this, but it may not be a good thing to allow. It's not a particularly good
    // attack surface, though, because of the low return.
    let mut n: N = Zero::zero();
    let mut new_n: N;
    let mut got_content = false;
    loop {
        match reader.read_byte() {
            Some(b) if b >= '0' as u8 && b <= '9' as u8 => {
                new_n = n * cast(10).unwrap() + cast(b).unwrap() - cast('0' as u8).unwrap();
            },
            Some(b) if got_content && expected_end(b) => return Some(n),
            _ => return None,
        }
        if new_n < n {
            // new_n < n implies overflow which is similarly not permitted.
            return None
        }
        n = new_n;
        got_content = true;
    }
}

/**
 * Read a positive hexadecimal integer from the given reader.
 *
 * # Arguments
 *
 * - `reader` - the reader to read the hexadecimal digits from (and whatever byte comes next)
 * - `expected_end` - this function is called with the byte that follows the last hexadecimal digit
 *   and should return `true` if that byte is what is expected to follow the number.
 *
 * # Return value
 *
 * - `None`, if the number overflows;
 * - `None`, if there is a leading zero;
 * - `None`, if all the digits are read and the `expected_end` function is false
 *
 * Should everything work as designed (i.e. none of these conditions occur) a `Some` is returned.
 */
pub fn read_hexadecimal<R: Reader, N: Unsigned + NumCast + Ord>
                       (reader: &mut R, expected_end: |u8| -> bool)
                       -> Option<N> {
    let mut n: N = Zero::zero();
    let mut new_n: N;
    let mut got_content = false;
    loop {
        match reader.read_byte() {
            Some(b) if b >= '0' as u8 && b <= '9' as u8 => {
                new_n = n * cast(16).unwrap() + cast(b).unwrap() - cast('0' as u8).unwrap();
            },
            Some(b) if b >= 'a' as u8 && b <= 'f' as u8 => {
                new_n = n * cast(16).unwrap() + cast(b).unwrap() - cast('a' as u8 - 10).unwrap();
            },
            Some(b) if b >= 'A' as u8 && b <= 'F' as u8 => {
                new_n = n * cast(16).unwrap() + cast(b).unwrap() - cast('A' as u8 - 10).unwrap();
            },
            Some(b) if got_content && expected_end(b) => return Some(n),
            _ => return None,
        }
        if new_n < n {
            // new_n < n implies overflow which is similarly not permitted.
            return None
        }
        n = new_n;
        got_content = true;
    }
}

/**
 * Read an HTTP-Version (e.g. "HTTP/1.1") from a stream.
 *
 * # Arguments
 *
 * - `reader` - the reader to read the HTTP-Version from (and whatever byte comes next)
 * - `expected_end` - this function is called with the byte that follows the HTTP-Version
 *   and should return `true` if that byte is what is expected to follow the HTTP-Version.
 *
 * # Return value
 *
 * - `None`, if the HTTP-Version is malformed in any way;
 * - `None`, if the `expected_end` function returns false;
 * - A `Some`, if all goes well.
 */
#[inline]
pub fn read_http_version<R: Reader>
                        (reader: &mut R, expected_end: |u8| -> bool)
                        -> Option<(uint, uint)> {
    let mut buf = [0u8, ..5];
    reader.read(buf);
    if (buf[0] != 'h' as u8 && buf[0] != 'H' as u8) ||
       (buf[1] != 't' as u8 && buf[1] != 'T' as u8) ||
       (buf[2] != 't' as u8 && buf[2] != 'T' as u8) ||
       (buf[3] != 'p' as u8 && buf[3] != 'P' as u8) ||
       buf[4] != '/' as u8 {
        return None;
    }

    let n1 = match read_decimal(reader, |b| b == '.' as u8) {
        Some(n) => n,
        None => return None,
    };
    let n2 = match read_decimal(reader, expected_end) {
        Some(n) => n,
        None => return None,
    };
    Some((n1, n2))
}

// I couldn't think what to call it. Ah well. It's just trivial syntax sugar, anyway.
macro_rules! test_reads {
    ($func:ident $($value:expr => $expected:expr),*) => {{
        $(
            assert_eq!(
                concat_idents!(read_, $func)(&mut MemReader::new($value.as_bytes().into_owned()),
                                             |b| b == 0),
                $expected);
        )*
    }}
}

#[test]
fn test_read_http_version() {
    test_reads!(http_version
                "HTTP/25.17\0" => Some((25, 17)),
                "http/1.0\0" => Some((1, 0)),
                "http 1.0\0" => None,
                "HTTP/1.0.\0" => None,
                "HTTP/1.0.\0" => None
    );
}

#[test]
fn test_read_decimal() {
    test_reads!(decimal
                "0\0" => Some(0u),
                "0\0" => Some(0u8),
                "0\0" => Some(0u64),
                "9\0" => Some(9u8),
                "42\0" => Some(42u64),
                "0123456789\0" => Some(123456789u64),
                "00000000000000000000000031\0" => Some(31u64),

                // End of stream in middle of number
                "0" => None::<u64>,

                // No number
                "\0" => None::<u64>,

                // Invalid character
                "0a\0" => None::<u64>,

                // At overflow bounds
                "255\0" => Some(255u8),
                "256\0" => None::<u8>,
                "256\0" => Some(256u16)
    );
}

#[test]
fn test_read_hexadecimal() {
    test_reads!(hexadecimal
                "0\0" => Some(0u),
                "0\0" => Some(0u8),
                "0\0" => Some(0u64),
                "f\0" => Some(0xfu8),
                "42\0" => Some(0x42u64),
                "012345\0" => Some(0x12345u64),
                "00000000000000000000000031\0" => Some(0x31u64),
                "0123456789AbCdEf\0" => Some(0x123456789abcdefu64),

                // End of stream in middle of number
                "0" => None::<u64>,

                // No number
                "\0" => None::<u64>,

                // Invalid character
                "fg\0" => None::<u64>,

                // At overflow bounds
                "ff\0" => Some(0xffu8),
                "100\0" => None::<u8>,
                "100\0" => Some(0x100u16)
    );
}
