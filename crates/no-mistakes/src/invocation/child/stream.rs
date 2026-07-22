//! Bounded, line-oriented streaming of a child process's stdout — a sibling
//! to `command_output` for callers that must not buffer the whole output
//! (e.g. a multi-megabyte `git diff`). Backpressure comes from a bounded
//! `sync_channel`: once its capacity is full the reader thread's next `send`
//! blocks, which blocks its next `read()`, which eventually blocks the
//! child's own `write()` once the OS pipe buffer fills too — no unbounded
//! buffer is ever held on either side.

use super::process_tree::{configure_process_group, ProcessTree};
use super::{fold_cleanup_error, receive_reader, PipeReader, CLEANUP_TIMEOUT};
use crate::invocation::deadline::remaining_timeout;
use reading::{decode_line, line_too_long, read_bounded, read_chunks};
use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use wait_timeout::ChildExt;

mod reading;

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
    // Fail fast, matching `command_output`, if the invocation deadline has
    // already elapsed — never spawn a child with no time budget left.
    remaining_timeout()?;

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
    let stderr_reader = spawn_bounded_stderr_reader(stderr);

    if let Err(error) = drain_lines(&chunk_rx, max_line_bytes, &mut on_line) {
        let cleanup_error = process_tree.terminate(&mut child).err();
        let _ = child.wait_timeout(CLEANUP_TIMEOUT);
        // Wake any sender still blocked on the now-abandoned channel so its
        // thread can observe disconnection and exit rather than hang.
        drop(chunk_rx);
        let _ = receive_reader(&stderr_reader);
        return Err(fold_cleanup_error(error, cleanup_error));
    }

    let status = match wait_for_exit(&mut child, &process_tree) {
        Ok(status) => status,
        Err(error) => {
            let _ = receive_reader(&stderr_reader);
            return Err(error);
        }
    };
    // Deadline-bounded, matching `command_output`'s stderr handling: a
    // descendant that inherited the pipe and keeps it open past the direct
    // child's own exit must not hang this call past the invocation deadline.
    let stderr = match receive_reader(&stderr_reader) {
        Ok(stderr) => stderr,
        Err(error) => {
            let cleanup_error = process_tree.terminate(&mut child).err();
            let _ = child.wait_timeout(CLEANUP_TIMEOUT);
            return Err(fold_cleanup_error(error, cleanup_error));
        }
    };
    Ok(StreamOutcome { status, stderr })
}

/// Spawns the stderr reader as a channel-backed [`PipeReader`] (like
/// `command_output`'s `spawn_reader`) rather than a raw `JoinHandle`, so
/// waiting for it goes through `receive_reader`'s deadline-aware
/// `recv_timeout` instead of an unbounded `join()`. `read_bounded` never
/// itself errors — it always eventually returns whatever it collected — so
/// this always sends `Ok`.
fn spawn_bounded_stderr_reader(pipe: impl Read + Send + 'static) -> PipeReader {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(Ok(read_bounded(pipe, STDERR_CAP_BYTES)));
    });
    receiver
}

/// Wait for the child to exit within the remaining invocation deadline (or
/// indefinitely if there is none), terminating and reaping its process tree
/// on *any* failure — a deadline timeout, an OS-level `wait` error, or a
/// `remaining_timeout` error from the deadline elapsing while this call
/// itself was waiting. Every one of those must route through the same
/// cleanup as a normal timeout (matching `command_output`'s
/// `cleanup_wait_error`), or the child tree is left unmanaged.
fn wait_for_exit(
    child: &mut std::process::Child,
    process_tree: &ProcessTree,
) -> std::io::Result<ExitStatus> {
    let remaining = match remaining_timeout() {
        Ok(remaining) => remaining,
        Err(error) => return Err(terminate_and_reap(process_tree, child, error)),
    };
    match remaining {
        Some(remaining) => match child.wait_timeout(remaining) {
            Ok(Some(status)) => Ok(status),
            Ok(None) => Err(terminate_and_reap(
                process_tree,
                child,
                std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "no-mistakes command deadline elapsed while waiting for a child process",
                ),
            )),
            Err(error) => Err(terminate_and_reap(process_tree, child, error)),
        },
        None => child
            .wait()
            .map_err(|error| terminate_and_reap(process_tree, child, error)),
    }
}

fn terminate_and_reap(
    process_tree: &ProcessTree,
    child: &mut std::process::Child,
    error: std::io::Error,
) -> std::io::Error {
    let cleanup_error = process_tree.terminate(child).err();
    let _ = child.wait_timeout(CLEANUP_TIMEOUT);
    fold_cleanup_error(error, cleanup_error)
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
                // Apply the cap to the line content without its terminating
                // newline, matching the `max_line_bytes` doc comment.
                Some(pos) if pos > max_line_bytes => return Err(line_too_long(max_line_bytes)),
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

#[cfg(test)]
#[path = "stream/tests.rs"]
mod tests;
