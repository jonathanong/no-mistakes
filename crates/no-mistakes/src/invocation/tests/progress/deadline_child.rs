use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

pub(super) struct ReadableDeadlineChild {
    child: Child,
    reader: Option<JoinHandle<String>>,
    readiness: Receiver<()>,
    mode: &'static str,
    started: Instant,
    ready_after: Option<Duration>,
    acknowledged_after: Option<Duration>,
}

impl ReadableDeadlineChild {
    pub(super) fn spawn(helper: &str, mode: &'static str) -> Self {
        let started = Instant::now();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", helper, "--nocapture"])
            .env("NO_MISTAKES_READABLE_DEADLINE_MODE", mode)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stderr = child.stderr.take().unwrap();
        let (ready, readiness) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut pipe = BufReader::new(stderr);
            let mut captured = Vec::new();
            loop {
                let start = captured.len();
                match pipe.read_until(b'\n', &mut captured) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(error) => {
                        let _ = writeln!(captured, "\nfailed to read child stderr: {error}");
                        break;
                    }
                }
                let line = &captured[start..];
                let line = line.strip_suffix(b"\n").unwrap_or(line);
                let line = line.strip_suffix(b"\r").unwrap_or(line);
                if (mode == "progress" && line.starts_with(b"[progress] "))
                    || (mode != "progress" && line == b"watchdog ready")
                {
                    let _ = ready.send(());
                }
            }
            String::from_utf8_lossy(&captured).into_owned()
        });
        Self {
            child,
            reader: Some(reader),
            readiness,
            mode,
            started,
            ready_after: None,
            acknowledged_after: None,
        }
    }

    pub(super) fn run(mut self) -> Result<(ExitStatus, WatchdogOutput), String> {
        let status = self.wait_for_exit();
        let elapsed = self.started.elapsed();
        if status.is_err() {
            self.stop();
        }
        let stderr = self.join_stderr();
        let output = WatchdogOutput {
            mode: self.mode,
            elapsed,
            ready_after: self.ready_after,
            acknowledged_after: self.acknowledged_after,
            stderr,
        };
        match status {
            Ok(status) => Ok((status, output)),
            Err(error) => Err(format!("{error}\n{output}")),
        }
    }

    fn wait_for_exit(&mut self) -> Result<ExitStatus, String> {
        self.readiness
            .recv_timeout(Duration::from_secs(10))
            .map_err(|error| {
                format!("helper must publish readiness before arming its watchdog: {error}")
            })?;
        self.ready_after = Some(self.started.elapsed());
        if let Some(status) = self
            .child
            .try_wait()
            .map_err(|error| format!("failed to check whether the watchdog exited: {error}"))?
        {
            return Err(format!("premature exit: {status}"));
        }
        self.child
            .stdin
            .take()
            .unwrap()
            .write_all(&[1])
            .map_err(|error| format!("failed to acknowledge watchdog readiness: {error}"))?;
        self.acknowledged_after = Some(self.started.elapsed());
        self.child
            .wait_timeout(Duration::from_secs(4))
            .map_err(|error| format!("failed to wait for watchdog exit: {error}"))?
            .ok_or_else(|| "watchdog hung".to_owned())
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    fn join_stderr(&mut self) -> String {
        self.reader
            .take()
            .and_then(|reader| reader.join().ok())
            .unwrap_or_else(|| "<child stderr reader failed>".to_owned())
    }
}

impl Drop for ReadableDeadlineChild {
    fn drop(&mut self) {
        if self.reader.is_some() {
            self.stop();
            let _ = self.join_stderr();
        }
    }
}

#[derive(Debug)]
pub(super) struct WatchdogOutput {
    mode: &'static str,
    elapsed: Duration,
    ready_after: Option<Duration>,
    acknowledged_after: Option<Duration>,
    pub(super) stderr: String,
}

impl fmt::Display for WatchdogOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "mode={} elapsed_ms={} readiness={:?} acknowledgement={:?}\nchild stderr:\n{}",
            self.mode,
            self.elapsed.as_millis(),
            self.ready_after,
            self.acknowledged_after,
            self.stderr
        )
    }
}
