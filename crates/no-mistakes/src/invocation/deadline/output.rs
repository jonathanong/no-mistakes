//! A hard deadline must not wait for a full pipe or Rust's stdout lock.
use std::io::{self, Write};
use std::sync::mpsc;
use std::time::Duration;

pub(super) fn emit(text: String, output: crate::invocation::CliErrorOutput) {
    let (completed, receiver) = mpsc::channel();
    let _ = std::thread::Builder::new()
        .name("no-mistakes-timeout-json".to_string())
        .spawn(move || {
            if let Ok(mut writer) = duplicate_stdout() {
                let _ = writer.write_all(text.as_bytes());
                let _ = writer.flush();
            }
            output.complete();
            let _ = completed.send(());
        });
    // Detached OS writes are best-effort; cleanup always retains its bound.
    let _ = receiver.recv_timeout(Duration::from_millis(25));
}

#[cfg(unix)]
fn duplicate_stdout() -> io::Result<std::fs::File> {
    use std::os::fd::AsFd;
    std::io::stdout()
        .as_fd()
        .try_clone_to_owned()
        .map(std::fs::File::from)
}

#[cfg(windows)]
fn duplicate_stdout() -> io::Result<std::fs::File> {
    use std::os::windows::io::AsHandle;
    std::io::stdout()
        .as_handle()
        .try_clone_to_owned()
        .map(std::fs::File::from)
}

#[cfg(not(any(unix, windows)))]
fn duplicate_stdout() -> io::Result<impl Write> {
    Ok(std::io::stdout())
}
