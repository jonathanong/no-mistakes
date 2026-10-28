use super::*;
use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

#[path = "tests/timeout.rs"]
mod timeout;

#[cfg(unix)]
#[test]
fn command_error_subprocess_helper() {
    let Ok(mode) = std::env::var("NO_MISTAKES_PROGRESS_COMMAND_ERROR") else {
        return;
    };
    if mode == "closed" {
        // Closing stderr is confined to the child, before its writer is installed.
        unsafe { nix::libc::close(2) };
    } else {
        use std::io::Write;
        // The parent limits this child's pipe to 4096 bytes. Fill it before
        // snapshots block the worker and timings occupy its bounded queue.
        std::io::stderr().write_all(&[b'x'; 4096]).unwrap();
        std::io::stderr().flush().unwrap();
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-config/invalid/fixture");
    let cli = Cli::parse_from([
        "no-mistakes",
        "--progress",
        "--progress-interval",
        "3600000",
        "check",
        "--root",
        root.to_str().unwrap(),
        "--json",
    ]);
    let error = run(cli).expect_err("progress must preserve the authoritative command error");
    let message = format!("{error:#}");
    assert!(message.contains("failed to parse"), "{message}");
    assert!(message.contains("did not find expected key"), "{message}");
    assert_eq!(no_mistakes::invocation::timeout_exit_code(&error), None);
}

#[cfg(unix)]
#[test]
fn command_error_survives_unavailable_writer_and_occupied_progress_queue() {
    for mode in ["closed", "occupied"] {
        if mode == "occupied" && !cfg!(target_os = "linux") {
            continue;
        }
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "invocation::tests::command_error_subprocess_helper",
                "--nocapture",
            ])
            .env("NO_MISTAKES_PROGRESS_COMMAND_ERROR", mode)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(target_os = "linux")]
        if mode == "occupied" {
            use std::os::unix::process::CommandExt;
            // Only this subprocess's stderr pipe is resized before exec.
            unsafe {
                command.pre_exec(|| {
                    if nix::libc::fcntl(2, nix::libc::F_SETPIPE_SZ, 4096) == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }
        let mut child = command.spawn().unwrap();
        let status = child
            .wait_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap_or_else(|| {
                let _ = child.kill();
                let _ = child.wait();
                panic!("returning an error waited for progress stderr: {mode}");
            });
        let output = child.wait_with_output().unwrap();
        assert!(
            status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn json_output_is_detected_from_every_spelling() {
    let args = |list: &[&str]| list.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
    assert!(wants_json(args(&[
        "no-mistakes",
        "check",
        "--format",
        "json"
    ])));
    assert!(wants_json(args(&["no-mistakes", "check", "--format=json"])));
    assert!(wants_json(args(&["no-mistakes", "check", "--json"])));
    assert!(!wants_json(args(&[
        "no-mistakes",
        "check",
        "--format",
        "md"
    ])));
    assert!(!wants_json(args(&["no-mistakes", "check", "--format"])));
}

#[test]
fn lock_status_parses_as_its_own_command() {
    let cli = Cli::try_parse_from(["no-mistakes", "lock", "status", "--format", "json"]).unwrap();
    assert!(matches!(cli.command, super::super::Command::Lock(_)));
}
