//! Bounded, line-oriented streaming of a child process's stdout — a sibling
//! to `command_output` for callers that must not buffer the whole output
//! (e.g. a multi-megabyte `git diff`). Backpressure comes from a bounded
//! `sync_channel`: once its capacity is full the reader thread's next `send`
//! blocks, which blocks its next `read()`, which eventually blocks the
//! child's own `write()` once the OS pipe buffer fills too — no unbounded
//! buffer is ever held on either side.

use super::process_tree::{configure_process_group, ProcessTree};
use super::CLEANUP_TIMEOUT;
use crate::invocation::deadline::remaining_timeout;
use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, SyncSender};
use wait_timeout::ChildExt;

/// Bytes read per stdout chunk sent over the channel. Chosen to amortize
/// channel-send overhead (a 10 MB patch is ~160 sends instead of one send
/// per line) while keeping worst-case buffered memory small.
const CHUNK_BYTES: usize = 64 * 1024;
/// Bounded channel capacity: the reader thread blocks once this many chunks
/// are queued and unconsumed.
const CHUNK_QUEUE_CAPACITY: usize = 2;
/// Stderr is retained only up to this many bytes; the remainder is drained
/// (so the child can never deadlock writing to a full stderr pipe) but
/// discarded — a `git diff` failure's stderr is a short diagnostic line.
const STDERR_CAP_BYTES: usize = 64 * 1024;

#[derive(Debug)]
pub(crate) struct StreamOutcome {
    pub(crate) status: ExitStatus,
    pub(crate) stderr: Vec<u8>,
}

/// Run `command`, feeding each line of its stdout (without the trailing
/// newline, matching `str::lines()`) to `on_line` as it arrives. A line
/// (including any unterminated final line) longer than `max_line_bytes`
/// without a newline is rejected with an `InvalidData` error instead of
/// growing the buffer without bound — the pathological-line diagnostic.
/// On any error the child's whole process tree is terminated and reaped
/// before returning, matching `command_output`'s cancellation contract.
pub(crate) fn stream_command_lines(
    command: &mut Command,
    max_line_bytes: usize,
    mut on_line: impl FnMut(&str) -> std::io::Result<()>,
) -> std::io::Result<StreamOutcome> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_process_group(command);
    let mut child = command.spawn()?;

    #[cfg(unix)]
    let process_tree = ProcessTree::attach(&child);
    #[cfg(not(unix))]
    let process_tree = match ProcessTree::attach(&child) {
        Ok(process_tree) => process_tree,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait_timeout(CLEANUP_TIMEOUT);
            return Err(error);
        }
    };

    let stdout = child.stdout.take().expect("child stdout must be piped");
    let stderr = child.stderr.take().expect("child stderr must be piped");
    let (chunk_tx, chunk_rx) = mpsc::sync_channel(CHUNK_QUEUE_CAPACITY);
    std::thread::spawn(move || read_chunks(stdout, chunk_tx));
    let stderr_reader = std::thread::spawn(move || read_bounded(stderr, STDERR_CAP_BYTES));

    if let Err(error) = drain_lines(&chunk_rx, max_line_bytes, &mut on_line) {
        let cleanup_error = process_tree.terminate(&mut child).err();
        let _ = child.wait_timeout(CLEANUP_TIMEOUT);
        // Wake any sender still blocked on the now-abandoned channel so its
        // thread can observe disconnection and exit rather than hang.
        drop(chunk_rx);
        let _ = stderr_reader.join();
        return match cleanup_error {
            Some(cleanup_error) => Err(std::io::Error::new(
                error.kind(),
                format!("{error}; terminating the child process tree failed: {cleanup_error}"),
            )),
            None => Err(error),
        };
    }

    let status = match remaining_timeout()? {
        Some(remaining) => match child.wait_timeout(remaining)? {
            Some(status) => status,
            None => {
                let _ = process_tree.terminate(&mut child);
                let _ = child.wait_timeout(CLEANUP_TIMEOUT);
                let _ = stderr_reader.join();
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "no-mistakes command deadline elapsed while waiting for a child process",
                ));
            }
        },
        None => child.wait()?,
    };
    let stderr = stderr_reader.join().unwrap_or_default();
    Ok(StreamOutcome { status, stderr })
}

fn read_chunks(mut pipe: impl Read, tx: SyncSender<std::io::Result<Vec<u8>>>) {
    let mut buf = vec![0u8; CHUNK_BYTES];
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

fn read_bounded(mut pipe: impl Read, cap: usize) -> Vec<u8> {
    let mut buf = [0u8; CHUNK_BYTES];
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

/// Consume chunks from `rx`, splitting on `\n` (stripping a trailing `\r` so
/// CRLF patches parse the same as LF ones) and calling `on_line` per
/// complete line. Recomputes the deadline before every receive, mirroring
/// `child::receive_reader`, so a slow multi-chunk stream is bounded by the
/// invocation's total remaining time, not a single fixed read timeout.
fn drain_lines(
    rx: &Receiver<std::io::Result<Vec<u8>>>,
    max_line_bytes: usize,
    on_line: &mut impl FnMut(&str) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut pending: Vec<u8> = Vec::new();
    loop {
        let chunk = match remaining_timeout()? {
            Some(remaining) => match rx.recv_timeout(remaining) {
                Ok(chunk) => chunk,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "no-mistakes command deadline elapsed while reading child output",
                    ));
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            },
            None => match rx.recv() {
                Ok(chunk) => chunk,
                Err(_) => break,
            },
        };
        pending.extend_from_slice(&chunk?);
        loop {
            match pending.iter().position(|&b| b == b'\n') {
                // A completed line is still rejected if IT ALONE exceeds the
                // cap — bounding per-line memory means bounding the whole
                // line's length, not just the still-accumulating case below.
                // Checking only after this inner loop would miss a line that
                // crosses the cap and receives its terminating `\n` within
                // the same chunk (pending briefly holds the full oversized
                // line, but this loop drains and clears it before that
                // after-the-fact check ever runs).
                Some(pos) if pos + 1 > max_line_bytes => return Err(line_too_long(max_line_bytes)),
                Some(pos) => {
                    let line: Vec<u8> = pending.drain(..=pos).collect();
                    on_line(&decode_line(&line[..line.len() - 1]))?;
                }
                None => break,
            }
        }
        if pending.len() > max_line_bytes {
            return Err(line_too_long(max_line_bytes));
        }
    }
    // The in-loop check above already validates `pending` before every
    // `break`, so reaching here means any leftover unterminated line (an
    // EOF without a final newline) is already known to be within bounds.
    if !pending.is_empty() {
        on_line(&decode_line(&pending))?;
    }
    Ok(())
}

fn line_too_long(max_line_bytes: usize) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!(
            "git diff line exceeds {max_line_bytes} bytes without a newline; \
             malformed or pathological unified diff"
        ),
    )
}

fn decode_line(line: &[u8]) -> std::borrow::Cow<'_, str> {
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    String::from_utf8_lossy(line)
}

#[cfg(test)]
#[path = "stream/tests.rs"]
mod tests;
