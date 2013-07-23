/// Memory buffers for the benefit of `std::rt::io::net` which has slow read/write.

use std::rt::io::{Reader, Writer};
use std::cmp::min;
use std::ptr;

// 64KB chunks (moderately arbitrary)
static READ_BUF_SIZE: uint = 0x10000;
//static WRITE_BUF_SIZE: uint = 0x10000;
// TODO: consider removing constants and giving a buffer size in the constructor

struct BufferedReader<'self, T> {
    wrapped: &'self mut T,
    buffer: [u8, ..READ_BUF_SIZE],
    // The current position in the buffer
    pos: uint,
    // The last valid position in the reader
    max: uint,
}

impl<'self, T: Reader> BufferedReader<'self, T> {
    pub fn new<'a>(reader: &'a mut T/*, buffer_size: uint*/) -> BufferedReader<'a, T> {
        BufferedReader {
            wrapped: reader,
            buffer: [0u8, ..READ_BUF_SIZE], //[0u8, ..buffer_size],
            pos: 0u,
            max: 0u,
        }
    }

    /// Poke a single byte back so it will be read next. For this to make sense, you must have just
    /// read that byte. If `self.pos` is 0 and `self.max` is not 0 (i.e. if the buffer is just
    /// filled
    /// Very great caution must be used in calling this as it will fail if `self.pos` is 0.
    pub fn poke_byte(&mut self, byte: u8) {
        match (self.pos, self.max) {
            (0, 0) => self.max = 1,
            (0, _) => fail!("poke called when buffer is full"),
            (_, _) => self.pos -= 1,
        }
        self.buffer[self.pos] = byte;
    }
}

impl<'self, T: Reader> Reader for ~BufferedReader<'self, T> {
    /// Read at most N bytes into `buf`, where N is the minimum of `buf.len()` and the buffer size.
    ///
    /// At present, this makes no attempt to fill its buffer proactively, instead waiting until you
    /// ask.
    fn read(&mut self, buf: &mut [u8]) -> Option<uint> {
        if self.pos == self.max {
            // Run out of buffered content, read some more
            match self.wrapped.read(self.buffer) {
                None => {
                    self.pos = 0;
                    self.max = 0;
                    return None
                },
                Some(i) => {
                    self.pos = 0;
                    self.max = i;
                },
            }
        }
        let size = min(self.max - self.pos, buf.len());
        unsafe {
            do buf.as_mut_buf |p_dst, _len_dst| {
                do self.buffer.as_imm_buf |p_src, _len_src| {
                    // Note that copy_memory works on bytes; good, u8 is byte-sized
                    ptr::copy_memory(p_dst, ptr::offset(p_src, self.pos), size)
                }
            }
        }
        self.pos += size;
        Some(size)
    }

    /// Return whether the Reader has reached the end of the stream AND exhausted its buffer.
    fn eof(&mut self) -> bool {
        self.pos == self.max && self.wrapped.eof()
    }
}
