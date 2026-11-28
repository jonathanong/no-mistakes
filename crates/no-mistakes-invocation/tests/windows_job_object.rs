//! Windows Job Object regression: the child is created suspended, assigned to
//! a job, and must be resumed before `command_output` waits.

#![cfg(windows)]

use no_mistakes_invocation::{command_output, install_test_deadline};
use std::process::Command;
use std::time::Duration;

#[path = "windows_job_object/deadline_console.rs"]
mod deadline_console;

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
