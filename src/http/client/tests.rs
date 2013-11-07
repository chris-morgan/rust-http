#[cfg(test)];

use memstream::MemReaderFakeStream;
use client::request::RequestWriter;
use client::response::ResponseReader;

fn test() {
    let mut request = ~RequestWriter::new(Get, from_str("http://example.com/").unwrap());
    ResponseReader::construct(MemReaderFakeStream::new(bytes!("\
HTTP/1.1 200 OK\r\n\
ETag: W/\"it's an entity-tag!\"\r\n\
Content-Length: 28\r\n\
\r\n\
And here's the request body.")), request);
}
