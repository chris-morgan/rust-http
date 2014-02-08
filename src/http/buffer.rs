/// Memory buffers for the benefit of `std::io::net` which has slow read/write.

use std::io::{IoResult, Stream};
use std::cmp::min;
use std::vec;

// 64KB chunks (moderately arbitrary)
static READ_BUF_SIZE: uint = 0x10000;
static WRITE_BUF_SIZE: uint = 0x10000;
// TODO: consider removing constants and giving a buffer size in the constructor

pub struct BufferedStream<T> {
    wrapped: T,
    read_buffer: ~[u8],
    // The current position in the buffer
    read_pos: uint,
    // The last valid position in the reader
    read_max: uint,
    write_buffer: ~[u8],
    write_len: uint,

    writing_chunked_body: bool,
}

impl<T: Stream> BufferedStream<T> {
    pub fn new(stream: T) -> BufferedStream<T> {
        let mut read_buffer = vec::with_capacity(READ_BUF_SIZE);
        unsafe { read_buffer.set_len(READ_BUF_SIZE); }
        let mut write_buffer = vec::with_capacity(WRITE_BUF_SIZE);
        unsafe { write_buffer.set_len(WRITE_BUF_SIZE); }
        BufferedStream {
            wrapped: stream,
            read_buffer: read_buffer,
            read_pos: 0u,
            read_max: 0u,
            write_buffer: write_buffer,
            write_len: 0u,
            writing_chunked_body: false,
        }
    }
}

impl<T: Reader> BufferedStream<T> {
    /// Poke a single byte back so it will be read next. For this to make sense, you must have just
    /// read that byte. If `self.pos` is 0 and `self.max` is not 0 (i.e. if the buffer is just
    /// filled
    /// Very great caution must be used in calling this as it will fail if `self.pos` is 0.
    pub fn poke_byte(&mut self, byte: u8) {
        match (self.read_pos, self.read_max) {
            (0, 0) => self.read_max = 1,
            (0, _) => fail!("poke called when buffer is full"),
            (_, _) => self.read_pos -= 1,
        }
        self.read_buffer[self.read_pos] = byte;
    }

    #[inline]
    fn fill_buffer(&mut self) -> IoResult<()> {
        assert_eq!(self.read_pos, self.read_max);
        self.read_pos = 0;
        match self.wrapped.read(self.read_buffer) {
            Ok(i) => {
                self.read_max = i;
                Ok(())
            },
            Err(err) => {
                self.read_max = 0;
                Err(err)
            },
        }
    }

    /// Slightly faster implementation of read_byte than that which is provided by ReaderUtil
    /// (which just uses `read()`)
    #[inline]
    pub fn read_byte(&mut self) -> IoResult<u8> {
        if self.read_pos == self.read_max {
            // Fill the buffer, giving up if we've run out of buffered content
            if_ok!(self.fill_buffer());
        }
        self.read_pos += 1;
        Ok(self.read_buffer[self.read_pos - 1])
    }
}

impl<T: Writer> BufferedStream<T> {
    /// Finish off writing a response: this flushes the writer and in case of chunked
    /// Transfer-Encoding writes the ending zero-length chunk to indicate completion.
    ///
    /// At the time of calling this, headers MUST have been written, including the
    /// ending CRLF, or else an invalid HTTP response may be written.
    pub fn finish_response(&mut self) -> IoResult<()> {
        if_ok!(self.flush());
        if self.writing_chunked_body {
            if_ok!(self.wrapped.write(bytes!("0\r\n\r\n")));
        }
        Ok(())
    }
}

impl<T: Reader> Reader for BufferedStream<T> {
    /// Read at most N bytes into `buf`, where N is the minimum of `buf.len()` and the buffer size.
    ///
    /// At present, this makes no attempt to fill its buffer proactively, instead waiting until you
    /// ask.
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        if self.read_pos == self.read_max {
            // Fill the buffer, giving up if we've run out of buffered content
            if_ok!(self.fill_buffer());
        }
        let size = min(self.read_max - self.read_pos, buf.len());
        vec::bytes::copy_memory(buf, self.read_buffer.slice_from(self.read_pos).slice_to(size));
        self.read_pos += size;
        Ok(size)
    }
}

impl<T: Writer> Writer for BufferedStream<T> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        if buf.len() + self.write_len > self.write_buffer.len() {
            // This is the lazy approach which may involve multiple writes where it's really not
            // warranted. Maybe deal with that later.
            if self.writing_chunked_body {
                let s = format!("{}\r\n", (self.write_len + buf.len()).to_str_radix(16));
                if_ok!(self.wrapped.write(s.as_bytes()));
            }
            if self.write_len > 0 {
                if_ok!(self.wrapped.write(self.write_buffer.slice_to(self.write_len)));
                self.write_len = 0;
            }
            if_ok!(self.wrapped.write(buf));
            self.write_len = 0;
            if self.writing_chunked_body {
                if_ok!(self.wrapped.write(bytes!("\r\n")));
            }
        } else {
            unsafe { self.write_buffer.mut_slice_from(self.write_len).copy_memory(buf); }

            self.write_len += buf.len();
            if self.write_len == self.write_buffer.len() {
                if self.writing_chunked_body {
                    let s = format!("{}\r\n", self.write_len.to_str_radix(16));
                    if_ok!(self.wrapped.write(s.as_bytes()));
                    if_ok!(self.wrapped.write(self.write_buffer));
                    if_ok!(self.wrapped.write(bytes!("\r\n")));
                } else {
                    if_ok!(self.wrapped.write(self.write_buffer));
                }
                self.write_len = 0;
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> IoResult<()> {
        if self.write_len > 0 {
            if self.writing_chunked_body {
                let s = format!("{}\r\n", self.write_len.to_str_radix(16));
                if_ok!(self.wrapped.write(s.as_bytes()));
            }
            if_ok!(self.wrapped.write(self.write_buffer.slice_to(self.write_len)));
            if self.writing_chunked_body {
                if_ok!(self.wrapped.write(bytes!("\r\n")));
            }
            self.write_len = 0;
        }
        self.wrapped.flush()
    }
}
