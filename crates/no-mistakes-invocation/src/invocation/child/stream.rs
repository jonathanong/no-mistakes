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
use drain::drain_lines;
use reading::{read_bounded, read_chunks};
use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use wait_timeout::ChildExt;

#[path = "stream/drain.rs"]
mod drain;
#[path = "stream/reading.rs"]
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

type LineHandler<'a> = dyn FnMut(&str) -> std::io::Result<()> + 'a;

#[derive(Debug)]
pub struct StreamOutcome {
    pub status: ExitStatus,
    pub stderr: Vec<u8>,
}

/// Run `command`, feeding each line of its stdout (without the trailing
/// newline, matching `str::lines()`) to `on_line` as it arrives. A line
/// (including any unterminated final line) longer than `max_line_bytes`
/// without a newline is rejected with an `InvalidData` error instead of
/// growing the buffer without bound — the pathological-line diagnostic.
/// On any error the child's whole process tree is terminated and reaped
/// before returning, matching `command_output`'s cancellation contract.
pub fn stream_command_lines(
    command: &mut Command,
    max_line_bytes: usize,
    on_line: &mut LineHandler<'_>,
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

    let mut stdout = child.stdout.take().expect("child stdout must be piped");
    let stderr = child.stderr.take().expect("child stderr must be piped");
    let (chunk_tx, chunk_rx) = mpsc::sync_channel(CHUNK_QUEUE_CAPACITY);
    std::thread::spawn(move || read_chunks(&mut stdout, chunk_tx));
    let stderr_reader = spawn_bounded_stderr_reader(stderr);

    if let Err(error) = drain_lines(&chunk_rx, max_line_bytes, on_line) {
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
fn spawn_bounded_stderr_reader(mut pipe: impl Read + Send + 'static) -> PipeReader {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(Ok(read_bounded(&mut pipe, STDERR_CAP_BYTES)));
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
        None => match child.wait() {
            Ok(status) => Ok(status),
            Err(error) => Err(terminate_and_reap(process_tree, child, error)),
        },
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

// These tests intentionally exercise POSIX shell process and pipe behavior.
#[cfg(all(test, unix))]
#[path = "stream/tests.rs"]
mod tests;
