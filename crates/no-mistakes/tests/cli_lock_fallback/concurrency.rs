use super::{cli, isolated_command, success_json, ReadOnlyRuntime};
use crate::saved_fixture;
use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use wait_timeout::ChildExt;

const WORKER: &str = "cli_lock_fallback::concurrency::fallback_holder_worker";
const WORKER_ENV: &str = "NO_MISTAKES_FALLBACK_HOLDER";
const READY: &str = "fallback lock held";

#[test]
fn fallback_holder_worker() {
    if std::env::var_os(WORKER_ENV).is_none() {
        return;
    }
    let _guard = no_mistakes::invocation::InvocationGuard::acquire(Default::default()).unwrap();
    println!("{READY}");
    let mut input = [0];
    assert_eq!(std::io::stdin().read(&mut input).unwrap(), 0);
}

#[test]
fn concurrent_runs_serialize_on_the_fallback_lock() {
    let fixture = saved_fixture::materialize("invocation", "read-only-runtime");
    let root = fixture.path();
    let _runtime = ReadOnlyRuntime::new(root);
    let mut holder = ManagedChild(
        isolated_command(Command::new(std::env::current_exe().unwrap()), root)
            .args(["--exact", WORKER, "--quiet", "--nocapture"])
            .env(WORKER_ENV, "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let holder_ready = observe(holder.0.stdout.take().unwrap(), READY);
    holder_ready.recv_timeout(Duration::from_secs(10)).unwrap();

    let output = cli(root)
        .args(["lock", "status", "--format", "json"])
        .output()
        .unwrap();
    let status = success_json(&output);
    assert_eq!(status["state"], "held");
    assert_eq!(status["holder"]["pid"], holder.0.id());

    let mut waiting = ManagedChild(
        cli(root)
            .args(["check", "--root", root.join("repo").to_str().unwrap()])
            .args(["--format", "json", "--lock-timeout", "15"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let message = format!("waiting for lock held by pid {}", holder.0.id());
    let waiter_ready = observe(waiting.0.stderr.take().unwrap(), &message);
    waiter_ready.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(waiting.0.try_wait().unwrap().is_none());
    drop(holder.0.stdin.take());
    holder.assert_success();
    waiting.assert_success();
    let mut report = String::new();
    waiting
        .0
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut report)
        .unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&report).unwrap()["rules"].is_array());
}

fn observe(pipe: impl Read + Send + 'static, expected: &str) -> mpsc::Receiver<()> {
    let (send, receive) = mpsc::channel();
    let expected = expected.to_owned();
    std::thread::spawn(move || {
        let mut send = Some(send);
        for line in BufReader::new(pipe).lines() {
            if line.unwrap().starts_with(&expected) {
                if let Some(send) = send.take() {
                    let _ = send.send(());
                }
            }
        }
    });
    receive
}

struct ManagedChild(Child);

impl ManagedChild {
    fn assert_success(&mut self) {
        assert!(self
            .0
            .wait_timeout(Duration::from_secs(10))
            .unwrap()
            .unwrap()
            .success());
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[path = "process_deadline.rs"]
mod process_deadline;
