/// TODO: submit upstream

use std::rt::io::{Reader, Writer, Seek, SeekStyle};
use std::rt::io::mem::{MemReader, MemWriter};

/// Writes to an owned, growable byte vector but also implements read with fail-on-call methods.
struct MemWriterFakeStream(MemWriter);

impl MemWriterFakeStream {
    pub fn new() -> MemWriterFakeStream { MemWriterFakeStream(MemWriter::new()) }
}

impl Writer for MemWriterFakeStream {
    fn write(&mut self, buf: &[u8]) { (**self).write(buf) }
    fn flush(&mut self) { (**self).flush() }
}

impl Seek for MemWriterFakeStream {
    fn tell(&self) -> u64 { (**self).tell() }
    fn seek(&mut self, pos: i64, style: SeekStyle) { (**self).seek(pos, style) }
}

impl Reader for MemWriterFakeStream {
    fn read(&mut self, _buf: &mut [u8]) -> Option<uint> {
        fail!("Uh oh, you didn't aught to call MemWriterFakeStream.read()!")
    }
    fn eof(&mut self) -> bool {
        fail!("Uh oh, you didn't aught to call MemWriterFakeStream.eof()!")
    }
}

/// Reads from an owned byte vector, but also implements write with fail-on-call methods.
pub struct MemReaderFakeStream(MemReader);

impl MemReaderFakeStream {
    pub fn new(buf: ~[u8]) -> MemReaderFakeStream { MemReaderFakeStream(MemReader::new(buf)) }
}

impl Reader for MemReaderFakeStream {
    fn read(&mut self, buf: &mut [u8]) -> Option<uint> { (**self).read(buf) }
    fn eof(&mut self) -> bool { (**self).eof() }
}

impl Seek for MemReaderFakeStream {
    fn tell(&self) -> u64 { (**self).tell() }
    fn seek(&mut self, pos: i64, style: SeekStyle) { (**self).seek(pos, style) }
}

impl Writer for MemReaderFakeStream {
    fn write(&mut self, _buf: &[u8]) {
        fail!("Uh oh, you didn't aught to call MemReaderFakeStream.write()!")
    }
    fn flush(&mut self) {
        fail!("Uh oh, you didn't aught to call MemReaderFakeStream.flush()!")
    }
}

#[cfg(test)]
mod test {
    use super::{MemReaderFakeStream, MemWriterFakeStream};

    #[test]
    fn test_mem_writer_fake_stream() {
        let mut writer = MemWriterFakeStream::new();
        assert_eq!(writer.tell(), 0);
        writer.write([0]);
        assert_eq!(writer.tell(), 1);
        writer.write([1, 2, 3]);
        writer.write([4, 5, 6, 7]);
        assert_eq!(writer.tell(), 8);
    }

    #[test]
    fn test_mem_reader_fake_stream() {
        let mut reader = MemReaderFakeStream::new(~[0, 1, 2, 3, 4, 5, 6, 7]);
        let mut buf = [];
        assert_eq!(reader.read(buf), Some(0));
        assert_eq!(reader.tell(), 0);
        let mut buf = [0];
        assert_eq!(reader.read(buf), Some(1));
        assert_eq!(reader.tell(), 1);
        assert_eq!(buf, [0]);
        let mut buf = [0, ..4];
        assert_eq!(reader.read(buf), Some(4));
        assert_eq!(reader.tell(), 5);
        assert_eq!(buf, [1, 2, 3, 4]);
        assert_eq!(reader.read(buf), Some(3));
        assert_eq!(buf.slice(0, 3), [5, 6, 7]);
        assert!(reader.eof());
        assert_eq!(reader.read(buf), None);
        assert!(reader.eof());
    }
}
