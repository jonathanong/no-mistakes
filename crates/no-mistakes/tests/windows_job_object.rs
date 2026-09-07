//! Windows Job Object regression: the child is created suspended, assigned to
//! a job, and must be resumed before `command_output` waits.
//!
//! This lives in an integration test so native Windows CI can compile it
//! against the ordinary rlib instead of the `--cfg test` unit-test harness
//! that typechecks every lib test (~7,500 cases).

#![cfg(windows)]

use no_mistakes::invocation::{command_output, install_test_deadline};
use std::process::Command;
use std::time::Duration;

#[test]
fn command_output_resumes_child_after_job_assignment() {
    let _guard = install_test_deadline(Duration::from_secs(5)).unwrap();
    let mut command = Command::new("cmd.exe");
    command.args(["/D", "/C", "echo stdout&echo stderr>&2"]);

    let output = command_output(&mut command).unwrap();

    assert!(output.status.success());
    assert_eq!(output.stdout, b"stdout\r\n");
    assert_eq!(output.stderr, b"stderr\r\n");
}
