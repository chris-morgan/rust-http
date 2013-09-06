/// Memory buffers for the benefit of `std::rt::io::net` which has slow read/write.

use std::rt::io::{Reader, Writer, Stream};
use std::rt::io::net::tcp::TcpStream;
use std::cmp::min;
use std::ptr;
use common::read_uint;
use rfc2616::{CR, LF};

pub type BufTcpStream = BufferedStream<TcpStream>;

// 64KB chunks (moderately arbitrary)
static READ_BUF_SIZE: uint = 0x10000;
static WRITE_BUF_SIZE: uint = 0x10000;
// TODO: consider removing constants and giving a buffer size in the constructor

struct BufferedStream<T> {
    wrapped: T,
    read_buffer: [u8, ..READ_BUF_SIZE],
    // The current position in the buffer
    read_pos: uint,
    // The last valid position in the reader
    read_max: uint,
    write_buffer: [u8, ..WRITE_BUF_SIZE],
    write_len: uint,

    /// Some things being written may not like flush() being called yet (e.g. explicitly fail!())
    /// The BufferedReader may need to be flushed for good control, but let it provide for such
    /// cases by not calling the wrapped object's flush method in turn.
    call_wrapped_flush: bool,

    writing_chunked_body: bool,
}

impl<T: Stream> BufferedStream<T> {
    pub fn new(stream: T, call_wrapped_flush: bool) -> BufferedStream<T> {
        BufferedStream {
            wrapped: stream,
            read_buffer: [0u8, ..READ_BUF_SIZE],
            read_pos: 0u,
            read_max: 0u,
            write_buffer: [0u8, ..WRITE_BUF_SIZE],
            write_len: 0u,
            call_wrapped_flush: call_wrapped_flush,
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
    fn fill_buffer(&mut self) -> bool {
        assert_eq!(self.read_pos, self.read_max);
        match self.wrapped.read(self.read_buffer) {
            None => {
                self.read_pos = 0;
                self.read_max = 0;
                false
            },
            Some(i) => {
                self.read_pos = 0;
                self.read_max = i;
                true
            },
        }
    }

    /// Slightly faster implementation of read_byte than that which is provided by ReaderUtil
    /// (which just uses `read()`)
    #[inline]
    pub fn read_byte(&mut self) -> Option<u8> {
        if self.read_pos == self.read_max && !self.fill_buffer() {
            // Run out of buffered content, no more to come
            return None;
        }
        self.read_pos += 1;
        Some(self.read_buffer[self.read_pos - 1])
    }
}

impl<T: Writer> BufferedStream<T> {
    /// Finish off writing a response: this flushes the writer and in case of chunked
    /// Transfer-Encoding writes the ending zero-length chunk to indicate completion.
    ///
    /// At the time of calling this, headers MUST have been written, including the
    /// ending CRLF, or else an invalid HTTP response may be written.
    pub fn finish_response(&mut self) {
        self.flush();
        if self.writing_chunked_body {
            self.wrapped.write(bytes!("0\r\n\r\n"));
        }
    }
}

impl<T: Reader> Reader for BufferedStream<T> {
    /// Read at most N bytes into `buf`, where N is the minimum of `buf.len()` and the buffer size.
    ///
    /// At present, this makes no attempt to fill its buffer proactively, instead waiting until you
    /// ask.
    fn read(&mut self, buf: &mut [u8]) -> Option<uint> {
        if self.read_pos == self.read_max && !self.fill_buffer() {
            // Run out of buffered content, no more to come
            return None;
        }
        let size = min(self.read_max - self.read_pos, buf.len());
        unsafe {
            do buf.as_mut_buf |p_dst, _len_dst| {
                do self.read_buffer.as_imm_buf |p_src, _len_src| {
                    // Note that copy_memory works on bytes; good, u8 is byte-sized
                    ptr::copy_memory(p_dst, ptr::offset(p_src, self.read_pos as int), size)
                }
            }
        }
        self.read_pos += size;
        Some(size)
    }

    /// Return whether the Reader has reached the end of the stream AND exhausted its buffer.
    fn eof(&mut self) -> bool {
        self.read_pos == self.read_max && self.wrapped.eof()
    }
}

impl<T: Writer> Writer for BufferedStream<T> {
    fn write(&mut self, buf: &[u8]) {
        if buf.len() + self.write_len > self.write_buffer.len() {
            // This is the lazy approach which may involve multiple writes where it's really not
            // warranted. Maybe deal with that later.
            if self.writing_chunked_body {
                let s = fmt!("%s\r\n", (self.write_len + buf.len()).to_str_radix(16));
                self.wrapped.write(s.as_bytes());
            }
            if self.write_len > 0 {
                self.wrapped.write(self.write_buffer.slice_to(self.write_len));
                self.write_len = 0;
            }
            self.wrapped.write(buf);
            self.write_len = 0;
            if self.writing_chunked_body {
                self.wrapped.write(bytes!("\r\n"));
            }
        } else {
            // Safely copy buf onto the "end" of self.buffer
            unsafe {
                do buf.as_imm_buf |p_src, len_src| {
                    do self.write_buffer.as_mut_buf |p_dst, _len_dst| {
                        // Note that copy_memory works on bytes; good, u8 is byte-sized
                        ptr::copy_memory(ptr::mut_offset(p_dst, self.write_len as int),
                                         p_src, len_src)
                    }
                }
            }
            self.write_len += buf.len();
            if self.write_len == self.write_buffer.len() {
                if self.writing_chunked_body {
                    let s = fmt!("%s\r\n", self.write_len.to_str_radix(16));
                    self.wrapped.write(s.as_bytes());
                    self.wrapped.write(self.write_buffer);
                    self.wrapped.write(bytes!("\r\n"));
                } else {
                    self.wrapped.write(self.write_buffer);
                }
                self.write_len = 0;
            }
        }
    }

    fn flush(&mut self) {
        if self.write_len > 0 {
            if self.writing_chunked_body {
                let s = fmt!("%s\r\n", self.write_len.to_str_radix(16));
                self.wrapped.write(s.as_bytes());
            }
            self.wrapped.write(self.write_buffer.slice_to(self.write_len));
            if self.writing_chunked_body {
                self.wrapped.write(bytes!("\r\n"));
            }
            self.write_len = 0;
        }
        if self.call_wrapped_flush {
            self.wrapped.flush();
        }
    }
}

struct ChunkedReader<'self, R> {
    reader: &'self mut BufferedStream<R>,
    // Number of bytes remaining of the current chunk.
    // This INCLUDES the CRLF at the end of it.
    // The following guards apply when read() is not being called:
    // - 0 means no chunk current (possibly with ``self.finished == true``)
    // - 1 cannot occur
    // - 2 cannot occur
    // - N means a chunk of (N - 2) bytes.
    chunk_size: uint,
    finished: bool,
}

impl<'self, R: Reader> ChunkedReader<'self, R> {
    pub fn new(reader: &'self mut BufferedStream<R>) -> ChunkedReader<'self, R> {
        ChunkedReader {
            reader: reader,
            chunk_size: 0,
            finished: false,
        }
    }

    fn read_chunk_header(&mut self) -> Option<uint> {
        // 20 is the maximal size of uint on 64-bit platform. I REALLY don't like this way of doing
        // it. Why am I writing it?
        match read_uint(self.reader, 19, CR) {
            Some(n) => {
                if self.reader.read_byte() == Some(LF) {
                    Some(n)
                } else {
                    None
                }
            },
            None => None,
        }
    }
}

impl<'self, R: Reader> Reader for ChunkedReader<'self, R> {
    fn read(&mut self, buf: &mut [u8]) -> Option<uint> {
        if self.finished {
            return None;
        }
        if self.chunk_size == 0 {
            match self.read_chunk_header() {
                Some(0) | None => {
                    self.finished = true;
                    return None;
                },
                Some(n) => {
                    self.chunk_size = n + 2;
                }
            }
        }
        // Now I have a guarantee that self.chunk_size > 2. (The 2 being for the CR LF.)
        let buf_len = buf.len();
        let chunk_size_available = self.chunk_size - 2;
        match self.reader.read(buf.mut_slice_to(min(chunk_size_available, buf_len) + 1)) {
            Some(bytes_read) if bytes_read == chunk_size_available => {
                // Read all the chunk. Now ensure the CR LF is there.
                self.chunk_size = 0;
                if self.reader.read_byte() != Some(CR) || self.reader.read_byte() == Some(LF) {
                    // FIXME: raise a condition here.
                    self.finished = true;
                    None
                } else {
                    Some(bytes_read)
                }
            },
            Some(bytes_read) => {
                // Haven't read all the chunk
                self.chunk_size -= bytes_read;
                Some(bytes_read)
            },
            None => {
                self.finished = true;
                // FIXME: raise a condition here.
                None
            },
        }
    }

    fn eof(&mut self) -> bool {
        self.finished
    }
}
