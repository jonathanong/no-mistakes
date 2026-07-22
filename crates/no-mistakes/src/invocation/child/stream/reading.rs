//! Raw pipe-reading primitives for the streaming child helper: chunked
//! stdout reads feeding the bounded channel, the byte-capped stderr drain,
//! and line-decoding. Kept separate from `stream.rs`'s process-lifecycle
//! orchestration (spawn, wait, cleanup) since none of this depends on it.

use std::io::Read;
use std::sync::mpsc::SyncSender;

pub(super) fn read_chunks(mut pipe: impl Read, tx: SyncSender<std::io::Result<Vec<u8>>>) {
    let mut buf = vec![0u8; super::CHUNK_BYTES];
    loop {
        match pipe.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if tx.send(Ok(buf[..n].to_vec())).is_err() {
                    break;
                }
            }
            Err(error) => {
                let _ = tx.send(Err(error));
                break;
            }
        }
    }
}

pub(super) fn read_bounded(mut pipe: impl Read, cap: usize) -> Vec<u8> {
    let mut buf = [0u8; super::CHUNK_BYTES];
    let mut collected = Vec::new();
    loop {
        match pipe.read(&mut buf) {
            Ok(0) => break,
            Ok(n) if collected.len() < cap => {
                let take = (cap - collected.len()).min(n);
                collected.extend_from_slice(&buf[..take]);
            }
            Ok(_) => {} // already at cap; keep draining so the pipe can't back up
            Err(_) => break,
        }
    }
    collected
}

pub(super) fn line_too_long(max_line_bytes: usize) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!(
            "git diff line exceeds {max_line_bytes} bytes without a newline; \
             malformed or pathological unified diff"
        ),
    )
}

pub(super) fn decode_line(line: &[u8]) -> std::borrow::Cow<'_, str> {
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    String::from_utf8_lossy(line)
}
