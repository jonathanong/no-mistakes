//! A hard deadline must not wait for a full pipe or Rust's stdout lock.
use crate::invocation::CliErrorOutput;
use std::io;
use std::sync::mpsc;
use std::time::Duration;
#[path = "output/stdio.rs"]
mod stdio;

type WriterTask = Box<dyn FnOnce() + Send>;

pub(super) fn emit(text: String, output: CliErrorOutput) {
    emit_with_spawner(text, output, |task| {
        std::thread::Builder::new()
            .name("no-mistakes-timeout-json".to_string())
            .spawn(task)
            .map(|_| ())
    });
}

fn emit_with_spawner(
    text: String,
    output: CliErrorOutput,
    spawn: impl FnOnce(WriterTask) -> io::Result<()>,
) {
    let (completed, receiver) = mpsc::channel();
    let writer_output = output.clone();
    let spawned = spawn(Box::new(move || {
        // Raw stdout handles bypass Rust's stdout buffer, so relative ordering
        // with buffered output is not guaranteed. Do not acquire Rust's stdout
        // lock to flush: a stalled writer may hold it indefinitely.
        let _ = stdio::write(&text);
        writer_output.complete();
        let _ = completed.send(());
    }));
    if spawned.is_err() {
        output.release_and_wait();
        return;
    }
    // Detached OS writes are best-effort; cleanup always retains its bound.
    let _ = receiver.recv_timeout(Duration::from_millis(25));
}

#[cfg(test)]
#[path = "output/tests.rs"]
mod tests;
