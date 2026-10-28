//! `lock status`, EX_TEMPFAIL exits, and JSON errors against a lock held by another process.
#![cfg(unix)]
use crate::saved_fixture;
use serde_json::Value;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn command(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(args)
        // The harness variable makes every process use a private lock; drop it so
        // both processes share the lock file derived from the temporary home.
        .env_remove("CARGO_BIN_EXE_no-mistakes")
        .env("HOME", home)
        .env("XDG_CACHE_HOME", home)
        .env("XDG_RUNTIME_DIR", home)
        .output()
        .expect("no-mistakes should run")
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

/// Hold the lock the way a running invocation does, with the saved holder record.
fn hold_lock(home: &Path, fixture: &str) -> File {
    let status = json(&command(home, &["lock", "status", "--format", "json"]));
    assert_eq!(status["state"], "free");
    let lock_file = PathBuf::from(status["lockFile"].as_str().unwrap());
    let source = saved_fixture::materialize("invocation", fixture);
    std::fs::copy(source.path().join("invocation.lock"), &lock_file).unwrap();
    let file = File::options().write(true).open(&lock_file).unwrap();
    file.lock().unwrap();
    file
}

#[test]
fn lock_status_reports_the_holder_recorded_in_the_lock_file() {
    let home = tempfile::tempdir().unwrap();
    let _held = hold_lock(home.path(), "lock-holder");
    let status = json(&command(
        home.path(),
        &["lock", "status", "--format", "json"],
    ));
    assert_eq!(status["state"], "held");
    assert_eq!(status["holder"]["pid"], 4242);
    assert_eq!(status["holder"]["cwd"], "/work/other-repo");
    assert_eq!(
        status["holder"]["command"],
        "no-mistakes check --format json"
    );
    assert!(status["holder"]["ageSeconds"].as_u64().unwrap() > 0);
    let text = command(home.path(), &["lock", "status"]);
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(
        text.contains("pid 4242") && text.contains("/work/other-repo"),
        "{text}"
    );
}

#[test]
fn lock_status_is_free_once_the_holder_exits_even_with_a_stale_record() {
    let home = tempfile::tempdir().unwrap();
    drop(hold_lock(home.path(), "lock-holder"));
    let status = json(&command(
        home.path(),
        &["lock", "status", "--format", "json"],
    ));
    assert_eq!(status["state"], "free");
    assert!(status["holder"].is_null());
}

#[test]
fn lock_status_reads_a_legacy_pid_only_record() {
    let home = tempfile::tempdir().unwrap();
    let _held = hold_lock(home.path(), "lock-holder-legacy");
    let status = json(&command(
        home.path(),
        &["lock", "status", "--format", "json"],
    ));
    assert_eq!(status["holder"]["pid"], 264);
    assert!(status["holder"]["cwd"].is_null());
}

#[test]
fn fail_on_lock_exits_75_with_a_json_error_naming_the_holder() {
    let home = tempfile::tempdir().unwrap();
    let _held = hold_lock(home.path(), "lock-holder");
    let output = command(
        home.path(),
        &["check", "--fail-on-lock", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(75));
    let error = &json(&output)["error"];
    assert_eq!(error["kind"], "lock_busy");
    assert_eq!(error["phase"], "invocation.lock");
    assert_eq!(error["exitCode"], 75);
    assert_eq!(error["holder"]["pid"], 4242);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("pid 4242") && stderr.contains("/work/other-repo"),
        "{stderr}"
    );
}

#[test]
fn lock_timeout_exits_75_not_the_deadline_code() {
    let home = tempfile::tempdir().unwrap();
    let _held = hold_lock(home.path(), "lock-holder");
    let output = command(
        home.path(),
        &["check", "--lock-timeout", "1", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(75));
    let error = &json(&output)["error"];
    assert_eq!(error["kind"], "lock_timeout");
    assert_eq!(error["holder"]["pid"], 4242);
}

#[test]
fn lock_failures_keep_stdout_empty_without_json_output() {
    let home = tempfile::tempdir().unwrap();
    let _held = hold_lock(home.path(), "lock-holder");
    let output = command(home.path(), &["check", "--fail-on-lock"]);
    assert_eq!(output.status.code(), Some(75));
    assert!(output.stdout.is_empty());
}
