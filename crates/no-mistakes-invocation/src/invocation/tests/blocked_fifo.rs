#![expect(
    clippy::disallowed_methods,
    reason = "Deadline and instrumentation tests require a real monotonic clock."
)]
use super::*;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::sync::mpsc;
use std::time::Instant;

const HELPER: &str = "invocation::tests::blocked_fifo::blocked_fifo_subprocess_helper";
const CONTROL: &str = "NO_MISTAKES_BLOCKED_FIFO_ROOT";

#[test]
fn blocked_fifo_subprocess_helper() {
    let Some(root) = std::env::var_os(CONTROL).map(PathBuf::from) else {
        return;
    };
    let fifo = root.join("blocked.yml");
    let reader_path = fifo.clone();
    let worker = std::thread::spawn(move || {
        set_timeout_phase("check");
        std::fs::read_to_string(reader_path)
    });
    // Retain a writer without data so the worker blocks inside the FIFO read.
    let _writer = OpenOptions::new().write(true).open(&fifo).unwrap();
    assert!(!worker.is_finished(), "FIFO read must remain blocked");
    println!("blocked fifo ready");
    std::io::stdout().flush().unwrap();
    let mut start = [0];
    std::io::stdin().read_exact(&mut start).unwrap();
    assert_eq!(start, [1]);
    let _execution = InvocationGuard::acquire_for_cli(InvocationOptions {
        timeout: Some(Duration::from_secs(1)),
        ..Default::default()
    })
    .unwrap();
    let outcome = worker.join().unwrap();
    panic!("watchdog failed to stop blocked FIFO read: {outcome:?}");
}

#[test]
fn watchdog_stops_blocked_fifo_read_and_names_check_phase() {
    let root = tempfile::tempdir().unwrap();
    assert!(Command::new("mkfifo")
        .arg(root.path().join("blocked.yml"))
        .status()
        .unwrap()
        .success());
    let mut child = ManagedChild(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", HELPER, "--nocapture"])
            .env(CONTROL, root.path())
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
            if line.unwrap() == "blocked fifo ready" {
                let _ = sender.send(());
                break;
            }
        }
    });
    receiver
        .recv_timeout(Duration::from_secs(15))
        .expect("worker must block on FIFO before its deadline is armed");
    reader.join().unwrap();
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
        panic!("blocked helper exited before start gate opened: {status}: {stderr}");
    }
    let started = Instant::now();
    start_gate.write_all(&[1]).unwrap();
    let status = child
        .0
        .wait_timeout(Duration::from_secs(8))
        .unwrap()
        .expect("blocked helper kept running past timeout backstop");
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
