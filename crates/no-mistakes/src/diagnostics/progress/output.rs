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

pub(in crate::diagnostics) struct ProgressOutput {
    sender: mpsc::SyncSender<Message>,
}

impl ProgressOutput {
    pub(in crate::diagnostics) fn stderr() -> Arc<Self> {
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

    pub(in crate::diagnostics) fn emit(&self, text: String) {
        if let Some(receiver) = self.try_enqueue(text) {
            let _ = receiver.recv_timeout(OUTPUT_WAIT);
        }
    }

    /// Queue one chunk without waiting on the OS write. `None` means the
    /// bounded queue was full or disconnected and the chunk was dropped.
    pub(in crate::diagnostics) fn try_enqueue(&self, text: String) -> Option<mpsc::Receiver<()>> {
        let (completed, receiver) = mpsc::channel();
        self.sender
            .try_send(Message { text, completed })
            .ok()
            .map(|()| receiver)
    }

    pub(in crate::diagnostics) fn flush_enqueued(&self, receiver: mpsc::Receiver<()>) {
        let _ = receiver.recv_timeout(OUTPUT_WAIT);
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
mod tests;
