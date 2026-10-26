//! CLI-only best-effort output. No caller waits on a potentially blocked pipe.
#[cfg(any(unix, windows))]
use std::fs::File;
use std::io::{self, Write};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

const OUTPUT_WAIT: Duration = Duration::from_millis(25);
const SIGNAL_OUTPUT_WAIT: Duration = Duration::from_millis(250);
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

    /// Give a termination snapshot one total bounded chance to enter and drain
    /// the same writer queue, even if a heartbeat currently occupies it.
    pub(in crate::diagnostics) fn emit_signal(&self, text: String, clock: impl Fn() -> Instant) {
        let deadline = clock() + SIGNAL_OUTPUT_WAIT;
        let (completed, receiver) = mpsc::channel();
        let mut message = Message { text, completed };
        loop {
            match self.sender.try_send(message) {
                Ok(()) => {
                    let _ = receiver.recv_timeout(deadline.saturating_duration_since(clock()));
                    return;
                }
                Err(mpsc::TrySendError::Full(pending)) => message = pending,
                Err(mpsc::TrySendError::Disconnected(_)) => return,
            }
            let remaining = deadline.saturating_duration_since(clock());
            if remaining.is_zero() {
                return;
            }
            std::thread::sleep(remaining.min(Duration::from_millis(1)));
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
