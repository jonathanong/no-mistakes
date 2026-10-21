use super::*;
use crate::diagnostics::InvocationGuard;
use crate::invocation::{InvocationGuard as ExecutionGuard, InvocationOptions};
use std::process::{Command, Stdio};
use wait_timeout::ChildExt;

#[test]
fn held_snapshot_lock_subprocess_helper() {
    let Ok(mode) = std::env::var("NO_MISTAKES_TIMEOUT_HELD_SNAPSHOT") else {
        return;
    };
    let observer = InvocationObserver::with_progress(true, mode == "progress");
    observer.increment("source.reads", 7);
    let _scope = InvocationGuard::install(Arc::clone(&observer));
    // Hold each snapshot lock across expiry. The watchdog must bound snapshot
    // collection itself, as well as writes to a blocked diagnostic pipe.
    let _timings = (mode == "timings").then(|| observer.timings.lock().unwrap());
    let _progress = observer
        .progress
        .as_ref()
        .map(|state| state.lock().unwrap());
    let _execution = ExecutionGuard::acquire_for_cli(InvocationOptions {
        timeout: Some(Duration::from_millis(150)),
        invocation_timeout: None,
        ..InvocationOptions::default()
    })
    .unwrap();
    std::thread::sleep(Duration::from_secs(10));
    panic!("snapshot lock prevented watchdog exit");
}

#[test]
fn watchdog_exits_when_timing_or_progress_snapshot_lock_is_held() {
    for mode in ["timings", "progress"] {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "diagnostics::observer::timeout_tests::held_snapshot_lock_subprocess_helper",
                "--nocapture",
            ])
            .env("NO_MISTAKES_TIMEOUT_HELD_SNAPSHOT", mode)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let status = child
            .wait_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap_or_else(|| {
                let _ = child.kill();
                let _ = child.wait();
                panic!("snapshot lock blocked hard deadline: {mode}");
            });
        let output = child.wait_with_output().unwrap();
        assert_eq!(status.code(), Some(124), "{mode}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("command timed out"), "{stderr}");
        assert!(
            stderr.contains("[PARTIAL work] source.reads: 7\n"),
            "{stderr}"
        );
        if mode == "timings" {
            assert!(stderr.contains("completed timings unavailable"), "{stderr}");
        }
    }
}
