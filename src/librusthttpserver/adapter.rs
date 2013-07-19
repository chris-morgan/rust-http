//! Code to adapt between the outgoing `std::io` and the incoming `std::rt::io` traits.
//! This code will be rendered obsolete before Rust 1.0.

use std::comm::{Port, Chan, SharedChan};
use std::{rt, task, vec};
use extra::{net, uv_iotask};

static READ_SIZE : uint = 1024;

/// Adapter of extra::net stuff to std::rt::io::net stuff
pub struct ExtraNetTcpListener {
    accept_ch: Chan<()>,
    stream_po: Port<Option<ExtraNetTcpStream>>,
}

enum StreamOperationRequest { Read(uint), Write }

impl ExtraNetTcpListener {
    pub fn bind(addr: rt::io::net::ip::IpAddr) -> Option<ExtraNetTcpListener> {
        match addr {
            rt::io::net::ip::Ipv4(a, b, c, d, p) => {
                let (accept_po, accept_ch) = stream();
                let (stream_po, stream_ch) = stream();
                let stream_ch = SharedChan::new(stream_ch);
                do spawn {
                    net::tcp::listen(
                        net::ip::v4::parse_addr(fmt!("%?.%?.%?.%?", a, b, c, d)),
                        p as uint,
                        100,
                        &uv_iotask::spawn_iotask(task::task()),
                        |_kill_ch| { },
                        |new_conn, _kill_ch| {
                            // An accept request has been sent, try accepting it
                            accept_po.recv();
                            let child_stream_ch = stream_ch.clone();
                            do task::spawn_supervised {
                                let accept_result = net::tcp::accept(new_conn);
                                match accept_result {
                                    Ok(socket) => {
                                        let (req_po, req_ch) = stream();
                                        let (write_po, write_ch) = stream();
                                        let (read_po, read_ch) = stream();
                                        let sockbuf = net::tcp::socket_buf(socket);
                                        let stream = ExtraNetTcpStream {
                                            req_ch: req_ch,
                                            write_ch: write_ch,
                                            read_po: read_po,
                                        };
                                        child_stream_ch.send(Some(stream));
                                        loop {
                                            match req_po.try_recv() {
                                                Some(Read(1)) => {
                                                    match sockbuf.read_byte() {
                                                        v if v >= 0 && v <= 255 => {
                                                            read_ch.send((~[v as u8], 1u));
                                                        },
                                                        _ => {
                                                            read_ch.send((~[0u8], 0u));
                                                        }
                                                    }
                                                },
                                                Some(Read(len)) => {
                                                    //let mut buf = vec::with_capacity(len);
                                                    let mut buf = vec::from_elem(len, 0u8);
                                                    let size = sockbuf.read(buf, len);
                                                    //let mut buf = ~[0u8, ..READ_SIZE];
                                                    //let size = sockbuf.read(buf, READ_SIZE);
                                                    read_ch.send((buf, size))
                                                },
                                                Some(Write) => {
                                                    sockbuf.write(write_po.recv());
                                                }
                                                None => {
                                                    // Closed: request completed
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        debug!("Error accepting, %?", err);
                                        child_stream_ch.send(None);
                                    }
                                }
                            }
                        }
                    );
                }

                Some(ExtraNetTcpListener {
                    accept_ch: accept_ch,
                    stream_po: stream_po,
                })
            },
            rt::io::net::ip::Ipv6(_, _, _, _, _, _, _, _, _port) => fail!("no IPv6 support")
        }
    }
}

impl rt::io::Listener<ExtraNetTcpStream> for ExtraNetTcpListener {
    pub fn accept(&mut self) -> Option<ExtraNetTcpStream> {
        self.accept_ch.send(());

        match self.stream_po.try_recv() {
            Some(optstream) => optstream,
            None => None,
        }
    }
}

pub struct ExtraNetTcpStream {
    req_ch: Chan<StreamOperationRequest>,
    write_ch: Chan<~[u8]>,
    read_po: Port<(~[u8], uint)>,
}

impl rt::io::Reader for ExtraNetTcpStream {
    //io::Reader has fn read(&self, buf: &mut [u8], len: uint) -> uint
    fn read(&mut self, buf: &mut [u8]) -> Option<uint> {
        self.req_ch.send(Read(buf.len()));
        let (read_buf, num_bytes_read) = self.read_po.recv();
        let mut i = 0;
        while i < num_bytes_read {
            buf[i] = read_buf[i];
            i += 1;
        }
        return Some(num_bytes_read);
    }

    fn eof(&mut self) -> bool { fail!("ExtraNetTcpStream.eof() is not supported") }
}

impl rt::io::Writer for ExtraNetTcpStream {
    fn write(&mut self, buf: &[u8]) {
        self.req_ch.send(Write);
        self.write_ch.send(buf.to_owned())
    }

    fn flush(&mut self) { fail!("ExtraNetTcpStream.flush() is not supported") }
}
