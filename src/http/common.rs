//! Schizofrenic methods which are useful in more than one of request-or-response reading-or-writing
use std::rt::io::Reader;

pub fn read_uint<R: Reader>(reader: &mut R, max_digits: u8, next_u8: u8) -> Option<uint> {
    let mut digits = 0u8;
    let mut n = 0u;
    loop {
        if digits == max_digits + 1 {
            return None;
        }
        match reader.read_byte() {
            Some(b) if b >= '0' as u8 && b <= '9' as u8 => {
                n = n * 10 + b as uint - '0' as uint;
            },
            Some(b) if b == next_u8 => return Some(n),
            _ => return None,
        }
        digits += 1;
    }
}

#[inline]
pub fn read_http_version<T: Reader>(reader: &mut T, next_u8: u8) -> Option<(uint, uint)> {
    // XXX: by doing this, I've stopped the more efficient BufferedStream.read_byte from being used
    // HTTP/%u.%u
    match reader.read_byte() {
        Some(b) if b == 'h' as u8 || b == 'H' as u8 => (),
        _ => return None,
    }
    match reader.read_byte() {
        Some(b) if b == 't' as u8 || b == 'T' as u8 => (),
        _ => return None,
    }
    match reader.read_byte() {
        Some(b) if b == 't' as u8 || b == 'T' as u8 => (),
        _ => return None,
    }
    match reader.read_byte() {
        Some(b) if b == 'p' as u8 || b == 'P' as u8 => (),
        _ => return None,
    }
    match reader.read_byte() {
        Some(b) if b == ('/' as u8) => (),
        _ => return None,
    }

    // First number
    let mut digits = 0u8;
    let mut n1 = 0u;
    loop {
        if digits == 5u8 {
            return None;
        }
        match reader.read_byte() {
            Some(b) if b >= '0' as u8 && b <= '9' as u8 => {
                n1 = n1 * 10 + b as uint - '0' as uint;
            },
            Some(b) if b == '.' as u8 => break,
            _ => return None,
        }
        digits += 1;
    }

    // Second number
    digits = 0u8;
    let mut n2 = 0u;
    loop {
        if digits == 5u8 {
            return None;
        }
        match reader.read_byte() {
            Some(b) if b >= '0' as u8 && b <= '9' as u8 => {
                n2 = n2 * 10 + b as uint - '0' as uint;
            },
            Some(b) if b == next_u8 => break,
            _ => return None,
        }
        digits += 1;
    }
    Some((n1, n2))
}
