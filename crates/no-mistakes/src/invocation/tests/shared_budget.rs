use super::*;
use std::io::{Read, Write};
use std::process::Stdio;
use wait_timeout::ChildExt;

// This exact test is also an isolated process whose stdin remains owned by its parent.
#[test]
fn watchdog_before_lock_path_subprocess_helper() {
    if std::env::var_os("NO_MISTAKES_BEFORE_LOCK_BUDGET_CONTROL").is_none() {
        return;
    }
    let result = InvocationGuard::acquire_resolving_path(
        InvocationOptions {
            timeout: None,
            invocation_timeout: Some(Duration::from_millis(150)),
            ..InvocationOptions::default()
        },
        true,
        || {
            std::io::stdout()
                .write_all(b"owned-path-resolution-ready\n")
                .unwrap();
            std::io::stdout().flush().unwrap();
            let mut byte = [0];
            std::io::stdin().read_exact(&mut byte).unwrap();
            panic!("parent must not release blocked path resolution as a passing timeout control");
        },
    );
    panic!(
        "watchdog did not stop blocked resolution: {}",
        result.is_ok()
    );
}

#[test]
fn invocation_watchdog_owns_blocked_acquisition_before_the_lock_path_is_resolved() {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "invocation::tests::shared_budget::watchdog_before_lock_path_subprocess_helper",
            "--nocapture",
        ])
        .env("NO_MISTAKES_BEFORE_LOCK_BUDGET_CONTROL", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let input = child.stdin.take().unwrap();
    let status = child
        .wait_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap_or_else(|| {
            let _ = child.kill();
            let _ = child.wait();
            panic!("forced backstop: native watchdog did not own acquisition");
        });
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert_eq!(status.code(), Some(124));
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("owned-path-resolution-ready"));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("150 milliseconds during invocation.lock"));
}

#[cfg(unix)]
#[test]
fn term_resistant_registered_child_subprocess_helper() {
    use std::os::unix::net::UnixStream;
    let Some(path) = std::env::var_os("NO_MISTAKES_SHARED_BUDGET_SOCKET") else {
        return;
    };
    unsafe {
        nix::libc::signal(nix::libc::SIGTERM, nix::libc::SIG_IGN);
    }
    let mut stream = UnixStream::connect(path).unwrap();
    writeln!(stream, "{}:{}", std::process::id(), unsafe {
        nix::libc::getppid()
    })
    .unwrap();
    let mut byte = [0];
    // Only the native watchdog's SIGKILL or the parent's independent failure
    // cleanup ends this read; no timing sleep manufactures readiness or exit.
    let _ = stream.read(&mut byte);
}

#[cfg(unix)]
#[test]
fn registered_child_budget_subprocess_helper() {
    let Some(root) = std::env::var_os("NO_MISTAKES_SHARED_BUDGET_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let _execution = InvocationGuard::acquire_at_path(
        InvocationOptions {
            timeout: None,
            invocation_timeout: Some(Duration::from_secs(1)),
            ..InvocationOptions::default()
        },
        &root.join("invocation.lock"),
        true,
    )
    .unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "invocation::tests::shared_budget::term_resistant_registered_child_subprocess_helper",
            "--nocapture",
        ])
        .env("NO_MISTAKES_SHARED_BUDGET_SOCKET", root.join("child.sock"));
    let result = command_output(&mut command);
    if let Err(error) = &result {
        if error.kind() == std::io::ErrorKind::TimedOut {
            eprintln!("native owned child deadline elapsed: {error}");
            std::process::exit(124);
        }
    }
    panic!(
        "deadline did not end owned blocked work: {}",
        result.is_ok()
    );
}

#[cfg(unix)]
#[test]
fn shared_deadline_closes_a_real_term_resistant_registered_child_channel() {
    use std::os::unix::net::{UnixListener, UnixStream};
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("child.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "invocation::tests::shared_budget::registered_child_budget_subprocess_helper",
            "--nocapture",
        ])
        .env("NO_MISTAKES_SHARED_BUDGET_ROOT", directory.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let wrapper_pid = child.id();
    let reader = match std::thread::Builder::new().spawn(move || -> std::io::Result<Vec<u8>> {
        let (mut stream, _) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes)?;
        Ok(bytes)
    }) {
        Ok(reader) => reader,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("observer startup failed: {error}");
        }
    };
    let (status, forced) = match child.wait_timeout(Duration::from_secs(4)) {
        Ok(Some(status)) => (Some(status), false),
        _ => {
            let _ = child.kill();
            (child.wait().ok(), true)
        }
    };
    // Wake an unaccepted listener on any startup failure. If the real child
    // connected, the observer's bounded error drops its stream to release it.
    let _ = UnixStream::connect(&socket);
    let observed = reader.join();
    let output = child.wait_with_output();
    assert!(!forced, "forced outer cleanup is not qualification");
    assert_eq!(status.unwrap().code(), Some(124));
    let bytes = observed
        .unwrap()
        .expect("owned child must close the channel without observer timeout fallback");
    let identity = String::from_utf8(bytes).unwrap();
    let (pid, parent) = identity
        .trim()
        .split_once(':')
        .expect("real child identity");
    assert!(pid.parse::<u32>().unwrap() > 0);
    assert_eq!(parent.parse::<u32>().unwrap(), wrapper_pid);
    let stderr = String::from_utf8(output.unwrap().stderr).unwrap();
    assert!(
        stderr.contains("command timed out after 1 seconds")
            || stderr.contains("native owned child deadline elapsed"),
        "{stderr}"
    );
}
