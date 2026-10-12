//! CLI-only best-effort output. No caller waits on a potentially blocked pipe.
#[cfg(any(unix, windows))]
use std::fs::File;
use std::io::{self, Write};
use std::sync::{mpsc, Arc};
use std::time::Duration;

const OUTPUT_WAIT: Duration = Duration::from_millis(25);
struct Message {
    text: String,
    completed: mpsc::Sender<()>,
}

pub(super) struct ProgressOutput {
    sender: mpsc::SyncSender<Message>,
}

impl ProgressOutput {
    pub(super) fn stderr() -> Arc<Self> {
        Self::start(duplicate_stderr())
    }

    fn start(writer: io::Result<impl Write + Send + 'static>) -> Arc<Self> {
        let (sender, receiver) = mpsc::sync_channel::<Message>(1);
        // Deliberately detached: a blocked OS write must never hold cleanup or
        // signal termination hostage. This worker owns bytes, never an observer.
        std::thread::spawn(move || {
            let Ok(mut writer) = writer else {
                return;
            };
            while let Ok(message) = receiver.recv() {
                let _ = writer.write_all(message.text.as_bytes());
                let _ = writer.flush();
                let _ = message.completed.send(());
            }
        });
        Arc::new(Self { sender })
    }

    pub(super) fn emit(&self, text: String) {
        let (completed, receiver) = mpsc::channel();
        if self.sender.try_send(Message { text, completed }).is_ok() {
            let _ = receiver.recv_timeout(OUTPUT_WAIT);
        }
    }
}

#[cfg(unix)]
fn duplicate_stderr() -> io::Result<File> {
    use std::os::fd::AsFd;
    // Duplicating the descriptor does not change its shared file status flags.
    // File writes also avoid holding Rust's global stderr mutex while blocked.
    std::io::stderr()
        .as_fd()
        .try_clone_to_owned()
        .map(File::from)
}

#[cfg(windows)]
fn duplicate_stderr() -> io::Result<File> {
    use std::os::windows::io::AsHandle;
    std::io::stderr()
        .as_handle()
        .try_clone_to_owned()
        .map(File::from)
}

#[cfg(not(any(unix, windows)))]
fn duplicate_stderr() -> io::Result<impl Write> {
    Ok(std::io::stderr())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blocked_writer_and_full_queue_never_block_emission_or_cleanup() {
        struct BlockedWriter {
            ready: mpsc::Sender<()>,
            release: mpsc::Receiver<()>,
        }
        impl Write for BlockedWriter {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                let _ = self.ready.send(());
                self.release.recv().unwrap();
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let (ready, started) = mpsc::channel();
        let (release, blocked) = mpsc::channel();
        let output = ProgressOutput::start(Ok(BlockedWriter {
            ready,
            release: blocked,
        }));
        let now = std::time::Instant::now();
        output.emit("first".to_string());
        started.recv_timeout(Duration::from_secs(2)).unwrap();
        output.emit("queued".to_string());
        output.emit("dropped".to_string());
        drop(output);
        assert!(now.elapsed() < Duration::from_secs(1));
        release.send(()).unwrap();
        started.recv_timeout(Duration::from_secs(2)).unwrap();
        release.send(()).unwrap();
    }

    #[test]
    fn failed_writer_is_best_effort() {
        let output = ProgressOutput::start(Err::<io::Sink, _>(io::Error::other("closed stderr")));
        output.emit("ignored".to_string());
    }
}
