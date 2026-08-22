use super::*;
use std::io::{self, Read};
use std::sync::mpsc;

struct FailRead;

impl Read for FailRead {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("fail"))
    }
}

struct OneChunkThenFail {
    sent: bool,
}

impl Read for OneChunkThenFail {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.sent {
            return Err(io::Error::other("fail"));
        }
        self.sent = true;
        buf[0] = b'x';
        Ok(1)
    }
}

struct AlwaysReady;

impl Read for AlwaysReady {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        buf[0] = b'a';
        Ok(1)
    }
}

struct FiniteBytes {
    data: &'static [u8],
}

impl Read for FiniteBytes {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.data.is_empty() {
            return Ok(0);
        }
        let n = self.data.len().min(buf.len());
        buf[..n].copy_from_slice(&self.data[..n]);
        self.data = &self.data[n..];
        Ok(n)
    }
}

#[test]
fn read_chunks_forwards_read_errors() {
    let (tx, rx) = mpsc::sync_channel(1);
    read_chunks(FailRead, tx);
    assert!(rx.recv().unwrap().is_err());
}

#[test]
fn read_chunks_stops_when_the_consumer_drops() {
    let (tx, rx) = mpsc::sync_channel(1);
    drop(rx);
    read_chunks(AlwaysReady, tx);
}

#[test]
fn read_bounded_drains_past_the_cap_and_stops_on_error() {
    let collected = read_bounded(OneChunkThenFail { sent: false }, 0);
    assert!(collected.is_empty());
}

#[test]
fn decode_line_strips_carriage_returns() {
    assert_eq!(decode_line(b"ok\r").as_ref(), "ok");
    assert_eq!(decode_line(b"ok").as_ref(), "ok");
    assert_eq!(line_too_long(4).kind(), io::ErrorKind::InvalidData);
}

#[test]
fn read_chunks_and_bounded_stop_at_eof_and_keep_bytes_under_cap() {
    let (tx, rx) = mpsc::sync_channel(4);
    read_chunks(FiniteBytes { data: b"ab" }, tx);
    assert_eq!(rx.recv().unwrap().unwrap(), b"ab");
    assert!(rx.recv().is_err());

    let collected = read_bounded(FiniteBytes { data: b"abcdef" }, 3);
    assert_eq!(collected, b"abc");
}
