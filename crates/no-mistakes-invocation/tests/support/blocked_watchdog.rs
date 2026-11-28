#![expect(
    clippy::disallowed_methods,
    reason = "Deadline and instrumentation tests require a real monotonic clock."
)]
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

pub(super) fn assert_blocked_watchdog(helper: &str, control: &str, ready: &'static str) {
    let root = tempfile::tempdir().unwrap();
    assert!(Command::new("mkfifo")
        .arg(root.path().join("blocked.yml"))
        .status()
        .unwrap()
        .success());
    let mut child = ManagedChild(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", helper, "--nocapture"])
            .env(control, root.path())
            .env(
                "CARGO_BIN_EXE_no-mistakes",
                std::env::current_exe().unwrap(),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let (sender, receiver) = mpsc::channel();
    let stdout = child.0.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if line.unwrap() == ready {
                sender.send(()).unwrap();
                break;
            }
        }
    });
    receiver
        .recv_timeout(Duration::from_secs(15))
        .expect("check must reach blocked config loading before its deadline is armed");
    reader.join().unwrap();
    // ChildExt::wait_timeout closes Child::stdin; retain the start gate separately.
    let mut start_gate = child.0.stdin.take().unwrap();
    if let Some(status) = child.0.wait_timeout(Duration::from_millis(1100)).unwrap() {
        let mut stderr = String::new();
        child
            .0
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        panic!("blocked check exited before the start gate opened: {status}: {stderr}");
    }
    let started = Instant::now();
    start_gate.write_all(&[1]).unwrap();
    let status = child
        .0
        .wait_timeout(Duration::from_secs(8))
        .unwrap()
        .expect("check kept running past the timeout backstop");
    let elapsed = started.elapsed();
    let mut stderr = String::new();
    child
        .0
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert_eq!(status.code(), Some(124), "{stderr}");
    assert!(
        stderr.contains("command timed out after 1 seconds during check"),
        "{stderr}"
    );
    assert!(elapsed >= Duration::from_millis(900), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(5), "{elapsed:?} {stderr}");
    assert!(child.0.try_wait().unwrap().is_some());
}

struct ManagedChild(Child);

impl Drop for ManagedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
