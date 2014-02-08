/// TODO: submit upstream

use std::io::{IoResult, Seek, SeekStyle};
use std::io::{MemReader, MemWriter};

/// Writes to an owned, growable byte vector but also implements read with fail-on-call methods.
struct MemWriterFakeStream(MemWriter);

impl MemWriterFakeStream {
    pub fn new() -> MemWriterFakeStream { MemWriterFakeStream(MemWriter::new()) }
}

impl Writer for MemWriterFakeStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        let &MemWriterFakeStream(ref mut s) = self;
        s.write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        let &MemWriterFakeStream(ref mut s) = self;
        s.flush()
    }
}

impl Seek for MemWriterFakeStream {
    fn tell(&self) -> IoResult<u64> {
        let &MemWriterFakeStream(ref s) = self;
        s.tell()
    }

    fn seek(&mut self, pos: i64, style: SeekStyle) -> IoResult<()> {
        let &MemWriterFakeStream(ref mut s) = self;
        s.seek(pos, style)
    }
}

impl Reader for MemWriterFakeStream {
    fn read(&mut self, _buf: &mut [u8]) -> IoResult<uint> {
        fail!("Uh oh, you didn't aught to call MemWriterFakeStream.read()!")
    }
}

/// Reads from an owned byte vector, but also implements write with fail-on-call methods.
pub struct MemReaderFakeStream(MemReader);

impl MemReaderFakeStream {
    pub fn new(buf: ~[u8]) -> MemReaderFakeStream { MemReaderFakeStream(MemReader::new(buf)) }
}

impl Reader for MemReaderFakeStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        let &MemReaderFakeStream(ref mut s) = self;
        s.read(buf)
    }
}

impl Seek for MemReaderFakeStream {
    fn tell(&self) -> IoResult<u64> {
        let &MemReaderFakeStream(ref s) = self;
        s.tell()
    }

    fn seek(&mut self, pos: i64, style: SeekStyle) -> IoResult<()> {
        let &MemReaderFakeStream(ref mut s) = self;
        s.seek(pos, style)
    }
}

impl Writer for MemReaderFakeStream {
    fn write(&mut self, _buf: &[u8]) -> IoResult<()> {
        fail!("Uh oh, you didn't aught to call MemReaderFakeStream.write()!")
    }
    fn flush(&mut self) -> IoResult<()> {
        fail!("Uh oh, you didn't aught to call MemReaderFakeStream.flush()!")
    }
}

#[cfg(test)]
mod test {
    use super::{MemReaderFakeStream, MemWriterFakeStream};

    #[test]
    fn test_mem_writer_fake_stream() {
        let mut writer = MemWriterFakeStream::new();
        assert_eq!(writer.tell(),              Ok(0));
        assert_eq!(writer.write([0]),          Ok(()));
        assert_eq!(writer.tell(),              Ok(1));
        assert_eq!(writer.write([1, 2, 3]),    Ok(()));
        assert_eq!(writer.write([4, 5, 6, 7]), Ok(()));
        assert_eq!(writer.tell(),              Ok(8));
    }

    #[test]
    fn test_mem_reader_fake_stream() {
        let mut reader = MemReaderFakeStream::new(~[0, 1, 2, 3, 4, 5, 6, 7]);
        let mut buf = [];
        assert_eq!(reader.read(buf),      Ok(0));
        assert_eq!(reader.tell(),         Ok(0));
        let mut buf = [0];
        assert_eq!(reader.read(buf),      Ok(1));
        assert_eq!(reader.tell(),         Ok(1));
        assert_eq!(buf,                   [0]);
        let mut buf = [0, ..4];
        assert_eq!(reader.read(buf),      Ok(4));
        assert_eq!(reader.tell(),         Ok(5));
        assert_eq!(buf,                   [1, 2, 3, 4]);
        assert_eq!(reader.read(buf),      Ok(3));
        assert_eq!(buf.slice(0, 3),       [5, 6, 7]);
        assert_eq!(reader.read(buf).ok(), None);
    }
}
