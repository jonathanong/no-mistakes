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
mod tests;
