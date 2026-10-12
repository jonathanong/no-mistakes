use super::*;

#[cfg(unix)]
#[test]
fn active_progress_signal_subprocess_helper() {
    let Ok(mode) = std::env::var("NO_MISTAKES_PROGRESS_SIGNAL_MODE") else {
        return;
    };
    use crate::diagnostics::{InvocationObserver, ProgressGuard, TimingKind};
    let observer = (mode != "plain").then(|| InvocationObserver::with_progress(false, true));
    let _scope = observer.as_ref().map(|observer| {
        crate::diagnostics::InvocationGuard::install(std::sync::Arc::clone(observer))
    });
    // The CLI profile deliberately disables deadlines; progress still needs
    // the shared signal listener so a killed hosted step can emit a snapshot.
    let _execution = InvocationGuard::acquire_for_cli(InvocationOptions {
        timeout: None,
        ..InvocationOptions::default()
    })
    .unwrap();
    let stop = || {
        if mode != "kill" {
            unsafe {
                nix::libc::raise(signal_hook::consts::SIGTERM);
            }
        }
        // The parent sends SIGKILL only after reading a flushed heartbeat.
        std::thread::sleep(Duration::from_secs(10));
        panic!("signal did not terminate subprocess");
    };
    if let Some(observer) = observer {
        let _heartbeat =
            ProgressGuard::install(std::sync::Arc::clone(&observer), Duration::from_millis(5));
        observer.trace("analysis.waiting", TimingKind::Parallel, stop);
    } else {
        stop();
    }
}

#[cfg(unix)]
#[test]
fn progress_survives_sigkill_and_sigterm_preserves_default_signal_exit_status() {
    use std::os::unix::process::ExitStatusExt;
    use std::process::Stdio;
    for mode in ["plain", "term", "kill"] {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "invocation::tests::progress::active_progress_signal_subprocess_helper",
                "--nocapture",
            ])
            .env("NO_MISTAKES_PROGRESS_SIGNAL_MODE", mode)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut captured_stderr = None;
        if mode == "kill" {
            use std::io::BufRead;
            let stderr = child.stderr.take().unwrap();
            let (ready, heartbeat) = std::sync::mpsc::channel();
            captured_stderr = Some(std::thread::spawn(move || {
                let mut captured = String::new();
                for line in std::io::BufReader::new(stderr).lines() {
                    let line = line.unwrap();
                    if line.starts_with("[progress] ") {
                        let _ = ready.send(());
                    }
                    captured.push_str(&line);
                    captured.push('\n');
                }
                captured
            }));
            if heartbeat.recv_timeout(Duration::from_secs(4)).is_err() {
                let _ = child.kill();
                let _ = child.wait();
                panic!("no bounded heartbeat before SIGKILL");
            }
            unsafe {
                nix::libc::kill(child.id() as i32, signal_hook::consts::SIGKILL);
            }
        }
        let status = child
            .wait_timeout(Duration::from_secs(4))
            .unwrap()
            .unwrap_or_else(|| {
                let _ = child.kill();
                let _ = child.wait();
                panic!("progress signal subprocess hung: {mode}");
            });
        let output = child.wait_with_output().unwrap();
        assert_eq!(
            status.signal(),
            Some(if mode == "kill" {
                signal_hook::consts::SIGKILL
            } else {
                signal_hook::consts::SIGTERM
            })
        );
        let stderr = captured_stderr
            .map(|reader| reader.join().unwrap())
            .unwrap_or_else(|| String::from_utf8(output.stderr).unwrap());
        let snapshots = stderr
            .lines()
            .filter_map(|line| line.strip_prefix("[progress] "))
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        if mode == "plain" {
            assert!(snapshots.is_empty(), "{stderr}");
        } else {
            assert!(!snapshots.is_empty(), "{stderr}");
            assert!(snapshots.iter().any(|snapshot| snapshot["active"]
                .as_array()
                .unwrap()
                .iter()
                .any(|span| span["phase"] == serde_json::json!(["analysis.waiting"]))));
        }
    }
}

#[cfg(unix)]
#[test]
fn blocked_progress_stderr_subprocess_helper() {
    let Ok(mode) = std::env::var("NO_MISTAKES_BLOCKED_PROGRESS_MODE") else {
        return;
    };
    use crate::diagnostics::{InvocationObserver, ProgressGuard, TimingKind};
    use std::io::Write;
    let observer = InvocationObserver::with_progress(false, true);
    let _scope = crate::diagnostics::InvocationGuard::install(std::sync::Arc::clone(&observer));
    let heartbeat =
        ProgressGuard::install(std::sync::Arc::clone(&observer), Duration::from_secs(3600));
    let _execution = InvocationGuard::acquire_for_cli(InvocationOptions {
        timeout: (mode == "timeout").then(|| Duration::from_millis(150)),
        ..InvocationOptions::default()
    })
    .unwrap();
    // Fill the parent's unread stderr pipe through the real output worker.
    observer.emit_progress_output("x".repeat(1_048_576));
    std::io::stdout().write_all(b"x").unwrap();
    std::io::stdout().flush().unwrap();
    if mode == "normal" {
        crate::diagnostics::print_stderr(format_args!("warning under progress"));
        observer.render_stderr();
        drop(heartbeat);
    } else {
        observer.trace("analysis.waiting", TimingKind::Parallel, || {
            std::thread::sleep(Duration::from_secs(10));
            panic!("blocked stderr prevented termination");
        });
    }
}

#[cfg(unix)]
#[test]
fn unread_full_stderr_never_blocks_progress_cleanup_signal_or_internal_timeout() {
    use std::io::Read;
    use std::os::unix::process::ExitStatusExt;
    use std::process::Stdio;
    for mode in ["normal", "term", "timeout"] {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "invocation::tests::progress::blocked_progress_stderr_subprocess_helper",
                "--nocapture",
            ])
            .env("NO_MISTAKES_BLOCKED_PROGRESS_MODE", mode)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        // Drain stdout for readiness without reading the deliberately full stderr.
        let stdout = child.stdout.take().unwrap();
        let (ready, marker) = std::sync::mpsc::channel();
        let stdout_reader = std::thread::spawn(move || {
            let mut stdout = stdout;
            let mut byte = [0];
            while stdout.read_exact(&mut byte).is_ok() {
                if byte == *b"x" {
                    let _ = ready.send(());
                }
            }
        });
        if marker.recv_timeout(Duration::from_secs(3)).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            panic!("blocked-stderr helper did not become ready");
        }
        if mode == "term" {
            unsafe {
                nix::libc::kill(child.id() as i32, signal_hook::consts::SIGTERM);
            }
        }
        let status = child
            .wait_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap_or_else(|| {
                let _ = child.kill();
                let _ = child.wait();
                panic!("unread progress stderr blocked {mode}");
            });
        if mode == "term" {
            assert_eq!(status.signal(), Some(signal_hook::consts::SIGTERM));
        } else {
            assert_eq!(status.code(), Some(if mode == "normal" { 0 } else { 124 }));
        }
        // Only read stderr after exit; draining it earlier would hide the bug.
        let output = child.wait_with_output().unwrap();
        stdout_reader.join().unwrap();
        assert!(!output.stderr.is_empty());
    }
}

#[cfg(unix)]
#[test]
fn progress_waiting_for_lock_subprocess_helper() {
    let Some(path) = std::env::var_os("NO_MISTAKES_PROGRESS_HELD_LOCK") else {
        return;
    };
    use crate::diagnostics::{InvocationObserver, ProgressGuard, TimingKind};
    let observer = InvocationObserver::with_progress(false, true);
    let _scope = crate::diagnostics::InvocationGuard::install(std::sync::Arc::clone(&observer));
    let _heartbeat =
        ProgressGuard::install(std::sync::Arc::clone(&observer), Duration::from_secs(3600));
    observer.trace("invocation.prepare", TimingKind::Serial, || {
        let _execution = InvocationGuard::acquire_at_path(
            InvocationOptions {
                timeout: None,
                ..InvocationOptions::default()
            },
            Path::new(&path),
            true,
        )
        .unwrap();
        panic!("externally held lock unexpectedly acquired");
    });
}

#[cfg(unix)]
#[test]
fn sigterm_while_waiting_for_lock_flushes_before_first_heartbeat() {
    use std::io::BufRead;
    use std::os::unix::process::ExitStatusExt;
    use std::process::Stdio;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("held.lock");
    let _lock = acquire_lock(&path, None, false).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "invocation::tests::progress::progress_waiting_for_lock_subprocess_helper",
            "--nocapture",
        ])
        .env("NO_MISTAKES_PROGRESS_HELD_LOCK", &path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let (ready, waiting) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut captured = String::new();
        for line in std::io::BufReader::new(stderr).lines() {
            let line = line.unwrap();
            if line.starts_with("waiting for lock") {
                let _ = ready.send(());
            }
            captured.push_str(&line);
            captured.push('\n');
        }
        captured
    });
    if waiting.recv_timeout(Duration::from_secs(3)).is_err() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("helper did not wait for held lock");
    }
    unsafe {
        nix::libc::kill(child.id() as i32, signal_hook::consts::SIGTERM);
    }
    let status = child
        .wait_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap_or_else(|| {
            let _ = child.kill();
            let _ = child.wait();
            panic!("lock-wait SIGTERM did not terminate");
        });
    assert_eq!(status.signal(), Some(signal_hook::consts::SIGTERM));
    let stderr = reader.join().unwrap();
    let snapshot = stderr
        .lines()
        .find_map(|line| line.strip_prefix("[progress] "))
        .unwrap();
    let snapshot: serde_json::Value = serde_json::from_str(snapshot).unwrap();
    assert_eq!(
        snapshot["active"][0]["phase"],
        serde_json::json!(["invocation.prepare"])
    );
}
