use super::*;
#[test]
fn blocked_writer_and_full_queue_never_block_emission_or_cleanup() {
    use crate::diagnostics::{InvocationGuard, InvocationObserver};
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
    let observer = InvocationObserver::with_progress(false, true);
    observer.progress.as_ref().unwrap().lock().unwrap().output = Some(Arc::clone(&output));
    let scope = InvocationGuard::install(Arc::clone(&observer));
    let now = std::time::Instant::now();
    assert!(observer.emit_progress_output("first".to_string()));
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(observer.emit_progress_output("queued".to_string()));
    assert!(observer.emit_progress_output("dropped".to_string()));
    warning_is_formatted_once_without_blocking_fallback();
    drop(scope);
    drop(observer);
    drop(output);
    assert!(now.elapsed() < Duration::from_secs(1));
    release.send(()).unwrap();
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    release.send(()).unwrap();
}

#[test]
fn failed_writer_is_best_effort() {
    use crate::diagnostics::{InvocationGuard, InvocationObserver};
    let output = ProgressOutput::start(Err::<io::Sink, _>(io::Error::other("closed stderr")));
    let observer = InvocationObserver::with_progress(false, true);
    observer.progress.as_ref().unwrap().lock().unwrap().output = Some(output);
    let _scope = InvocationGuard::install(Arc::clone(&observer));
    assert!(observer.emit_progress_output("ignored".to_string()));
    warning_is_formatted_once_without_blocking_fallback();
}

fn warning_is_formatted_once_without_blocking_fallback() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Warning(AtomicUsize);
    impl std::fmt::Display for Warning {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fetch_add(1, Ordering::SeqCst);
            formatter.write_str("optional warning")
        }
    }
    let warning = Warning(AtomicUsize::new(0));
    crate::diagnostics::print_stderr(format_args!("{warning}"));
    // The routed warning is formatted once. A delivery-failure fallback would
    // format it again for ordinary stderr, which could block the deadline path.
    assert_eq!(warning.0.load(Ordering::SeqCst), 1);
}

#[test]
fn installed_heartbeat_flushes_snapshots_timings_and_warnings_through_its_writer() {
    use crate::diagnostics::{InvocationGuard, InvocationObserver, ProgressGuard, TimingKind};
    struct Capture(mpsc::Sender<String>);
    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .send(String::from_utf8(bytes.to_vec()).unwrap())
                .unwrap();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let observer = InvocationObserver::with_progress(true, true);
    let _scope = InvocationGuard::install(Arc::clone(&observer));
    observer.increment("output.errors", 1);
    observer.trace("completed", TimingKind::Serial, || ());
    observer.trace("active", TimingKind::Parallel, || {
        let guard = ProgressGuard::install(Arc::clone(&observer), Duration::from_millis(10));
        let (sender, receiver) = mpsc::channel();
        // Replace the newly installed duplicated descriptor with a deterministic
        // writer before the first tick; capture bytes without a stderr pipe race.
        observer.progress.as_ref().unwrap().lock().unwrap().output =
            Some(ProgressOutput::start(Ok(Capture(sender))));
        for _ in 0..3 {
            let line = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
            let snapshot: serde_json::Value =
                serde_json::from_str(line.strip_prefix("[progress] ").unwrap()).unwrap();
            assert!(snapshot["active"].as_array().unwrap().iter().any(|span| {
                span["phase"] == serde_json::json!(["active"]) && span["parallel"] == true
            }));
        }
        drop(guard);
        while receiver.try_recv().is_ok() {}
        observer.render_stderr();
        let timings = receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(timings.contains("[timing] completed:"));
        assert!(timings.contains("[work] output.errors: 1"));
        crate::diagnostics::print_stderr(format_args!("warning: still actionable"));
        assert_eq!(
            receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
            "warning: still actionable\n"
        );
    });
}

#[cfg(unix)]
#[test]
fn closed_stderr_subprocess_helper() {
    if std::env::var_os("NO_MISTAKES_CLOSED_PROGRESS_STDERR").is_none() {
        return;
    }
    // Closing stderr is confined to a child; the parent harness keeps its fd.
    unsafe {
        nix::libc::close(2);
    }
    let output = ProgressOutput::stderr();
    output.emit("best effort with closed stderr".to_string());
}

#[cfg(unix)]
#[test]
fn unavailable_stderr_descriptor_does_not_block_progress_shutdown() {
    use std::process::{Command, Stdio};
    use wait_timeout::ChildExt;
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "diagnostics::progress::output::tests::closed_stderr_subprocess_helper",
            "--nocapture",
        ])
        .env("NO_MISTAKES_CLOSED_PROGRESS_STDERR", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let status = child
        .wait_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap_or_else(|| {
            let _ = child.kill();
            let _ = child.wait();
            panic!("closed-stderr progress hung");
        });
    assert!(status.success());
}

#[test]
fn signal_output_waits_for_a_delayed_writer_to_flush() {
    struct DelayedWriter {
        sent: mpsc::Sender<String>,
        pending: Vec<u8>,
    }
    impl Write for DelayedWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.pending.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            std::thread::sleep(Duration::from_millis(75));
            self.sent
                .send(String::from_utf8(std::mem::take(&mut self.pending)).unwrap())
                .unwrap();
            Ok(())
        }
    }
    let (sent, received) = mpsc::channel();
    let output = ProgressOutput::start(Ok(DelayedWriter {
        sent,
        pending: Vec::new(),
    }));
    output.emit_signal("signal snapshot".to_string(), Instant::now);
    assert_eq!(received.try_recv().unwrap(), "signal snapshot");
}

struct GatedWriter {
    started: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
    written: mpsc::Sender<String>,
    first: bool,
}

impl Write for GatedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.first {
            self.first = false;
            self.started.send(()).unwrap();
            self.release.recv().unwrap();
        }
        self.written
            .send(String::from_utf8(bytes.to_vec()).unwrap())
            .unwrap();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn gated_output() -> (
    Arc<ProgressOutput>,
    mpsc::Sender<()>,
    mpsc::Receiver<String>,
) {
    let (started, waiting) = mpsc::channel();
    let (release, blocked) = mpsc::channel();
    let (written, received) = mpsc::channel();
    let output = ProgressOutput::start(Ok(GatedWriter {
        started,
        release: blocked,
        written,
        first: true,
    }));
    output.emit("first".to_string());
    waiting.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(output.try_enqueue("queued".to_string()).is_some());
    (output, release, received)
}

#[test]
fn signal_output_retries_a_full_queue_then_uses_the_same_writer() {
    let (output, release, received) = gated_output();
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(75));
        release.send(()).unwrap();
    });
    output.emit_signal("signal".to_string(), Instant::now);
    releaser.join().unwrap();
    for expected in ["first", "queued", "signal"] {
        assert_eq!(
            received.recv_timeout(Duration::from_secs(1)).unwrap(),
            expected
        );
    }
}

#[test]
fn signal_output_full_queue_remains_bounded_when_writer_blocks() {
    let (output, release, received) = gated_output();
    let started = std::time::Instant::now();
    output.emit_signal("signal".to_string(), Instant::now);
    assert!(started.elapsed() < Duration::from_secs(1));
    release.send(()).unwrap();
    assert_eq!(
        received.recv_timeout(Duration::from_secs(1)).unwrap(),
        "first"
    );
    assert_eq!(
        received.recv_timeout(Duration::from_secs(1)).unwrap(),
        "queued"
    );
    assert!(received.recv_timeout(Duration::from_millis(50)).is_err());
}
