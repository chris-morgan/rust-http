/*!
 * Poorly categorised functions for reading things used in multiple places.
 *
 * (That is, typically, more than one of request-or-response reading-or-writing.)
 *
 * TODO: refactor all this to store things in more usefully categorised places.
 */
use std::num::{Zero, cast};
use std::io::{IoError, IoResult, OtherIoError};
#[cfg(test)]
use std::io::MemReader;

// XXX: IoError ain't a good representation of this.
fn bad_input() -> IoError {
    IoError {
        kind: OtherIoError,
        desc: "invalid number",
        detail: None,
    }
}

static ASCII_ZERO: u8 = '0' as u8;
static ASCII_NINE: u8 = '9' as u8;
static ASCII_LOWER_A: u8 = 'a' as u8;
static ASCII_LOWER_F: u8 = 'f' as u8;
static ASCII_UPPER_A: u8 = 'A' as u8;
static ASCII_UPPER_F: u8 = 'F' as u8;

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
pub fn read_decimal<R: Reader, N: Unsigned + NumCast + Ord + CheckedMul + CheckedAdd>
                   (reader: &mut R, expected_end: |u8| -> bool)
                   -> IoResult<N> {
    // Here and in `read_hexadecimal` there is the possibility of infinite sequence of zeroes. The
    // spec allows this, but it may not be a good thing to allow. It's not a particularly good
    // attack surface, though, because of the low return.
    let mut n: N = Zero::zero();
    let mut got_content = false;
    let ten: N = cast(10).unwrap();
    loop {
        n = match reader.read_byte() {
            Ok(b@ASCII_ZERO..ASCII_NINE) => {
                // Written sanely, this is: n * 10 + (b - '0'), but we avoid
                // (semantically unsound) overflow by using checked operations.
                // There is no need in the b - '0' part as it is safe.
                match n.checked_mul(&ten).and_then(
                        |n| n.checked_add(&cast(b - ASCII_ZERO).unwrap())) {
                    Some(new_n) => new_n,
                    None => return Err(bad_input()),  // overflow
                }
            },
            Ok(b) if got_content && expected_end(b) => return Ok(n),
            Ok(_) => return Err(bad_input()),  // not a valid number
            Err(err) => return Err(err),  // I/O error
        };
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
pub fn read_hexadecimal<R: Reader, N: Unsigned + NumCast + Ord + CheckedMul + CheckedAdd>
                       (reader: &mut R, expected_end: |u8| -> bool)
                       -> IoResult<N> {
    let mut n: N = Zero::zero();
    let mut got_content = false;
    let sixteen: N = cast(16).unwrap();
    loop {
        n = match reader.read_byte() {
            Ok(b@ASCII_ZERO..ASCII_NINE) => {
                match n.checked_mul(&sixteen).and_then(
                        |n| n.checked_add(&cast(b - ASCII_ZERO).unwrap())) {
                    Some(new_n) => new_n,
                    None => return Err(bad_input()),  // overflow
                }
            },
            Ok(b@ASCII_LOWER_A..ASCII_LOWER_F) => {
                match n.checked_mul(&sixteen).and_then(
                        |n| n.checked_add(&cast(b - ASCII_LOWER_A + 10).unwrap())) {
                    Some(new_n) => new_n,
                    None => return Err(bad_input()),  // overflow
                }
            },
            Ok(b@ASCII_UPPER_A..ASCII_UPPER_F) => {
                match n.checked_mul(&sixteen).and_then(
                        |n| n.checked_add(&cast(b - ASCII_UPPER_A + 10).unwrap())) {
                    Some(new_n) => new_n,
                    None => return Err(bad_input()),  // overflow
                }
            },
            Ok(b) if got_content && expected_end(b) => return Ok(n),
            Ok(_) => return Err(bad_input()),  // not a valid number
            Err(err) => return Err(err),  // I/O error
        };
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
                        -> IoResult<(uint, uint)> {
    // I'd read into a [0u8, ..5], but that buffer is not guaranteed to be
    // filled, so I must read it byte by byte to guarantee correctness.
    // (Sure, no sane person/library would send the first packet with "HTT"
    // and leave the "P/1.1" to the next packet, but it's *possible*.)
    let b0 = try!(reader.read_byte());
    let b1 = try!(reader.read_byte());
    let b2 = try!(reader.read_byte());
    let b3 = try!(reader.read_byte());
    let b4 = try!(reader.read_byte());
    if (b0 != 'h' as u8 && b0 != 'H' as u8) ||
       (b1 != 't' as u8 && b1 != 'T' as u8) ||
       (b2 != 't' as u8 && b2 != 'T' as u8) ||
       (b3 != 'p' as u8 && b3 != 'P' as u8) ||
       b4 != '/' as u8 {
        return Err(bad_input());
    }

    let major = try!(read_decimal(reader, |b| b == '.' as u8));
    let minor = try!(read_decimal(reader, expected_end));
    Ok((major, minor))
}

// I couldn't think what to call it. Ah well. It's just trivial syntax sugar, anyway.
macro_rules! test_reads {
    ($func:ident $($value:expr => $expected:expr),*) => {{
        $(
            assert_eq!(
                concat_idents!(read_, $func)(&mut MemReader::new($value.bytes().collect::<Vec<_>>()),
                                             |b| b == 0).ok(),
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
