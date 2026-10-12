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
