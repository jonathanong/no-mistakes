use super::reading::{decode_line, line_too_long};
use super::LineHandler;
use crate::invocation::deadline::remaining_timeout;
use std::sync::mpsc::{self, Receiver};

pub(super) fn drain_lines(
    rx: &Receiver<std::io::Result<Vec<u8>>>,
    max_line_bytes: usize,
    on_line: &mut LineHandler<'_>,
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
                // Bound complete lines before draining so oversized lines never slip past.
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
    if !pending.is_empty() {
        on_line(&decode_line(&pending))?;
    }
    Ok(())
}
