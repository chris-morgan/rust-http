//! Code to adapt between the outgoing `std::io` and the incoming `std::rt::io` traits.
//! This code will be rendered obsolete before Rust 1.0.

use std::io;
use std::rt;


/// A wrapper of an `std::io::Writer` into an `std::rt::io::Writer`.
pub struct WriterRtWriterAdapter<T> {
    priv writer: T,
}

pub fn WriterRtWriterAdapter<T: io::Writer>(writer: T) -> WriterRtWriterAdapter<T> {
    WriterRtWriterAdapter { writer: writer }
}

impl<T: io::Writer> rt::io::Writer for WriterRtWriterAdapter<T> {
    fn write(&mut self, buf: &[u8]) {
        self.writer.write(buf);
    }

    fn flush(&mut self) {
        self.writer.flush();
    }
}
