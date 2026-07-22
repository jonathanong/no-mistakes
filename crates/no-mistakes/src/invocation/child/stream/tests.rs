use super::*;
use std::sync::Mutex;

fn collect_lines(
    command: &mut Command,
    max_line_bytes: usize,
) -> std::io::Result<(Vec<String>, StreamOutcome)> {
    let lines = Mutex::new(Vec::new());
    let outcome = stream_command_lines(command, max_line_bytes, |line| {
        lines.lock().unwrap().push(line.to_string());
        Ok(())
    })?;
    Ok((lines.into_inner().unwrap(), outcome))
}

#[test]
fn streams_stdout_line_by_line() {
    let mut command = Command::new("printf");
    command.arg("a\\nb\\nc\\n");
    let (lines, outcome) = collect_lines(&mut command, 1024).unwrap();
    assert!(outcome.status.success());
    assert_eq!(lines, vec!["a", "b", "c"]);
}

#[test]
fn flushes_a_final_line_without_trailing_newline() {
    let mut command = Command::new("printf");
    command.arg("a\\nb");
    let (lines, _outcome) = collect_lines(&mut command, 1024).unwrap();
    assert_eq!(lines, vec!["a", "b"]);
}

#[test]
fn strips_carriage_return_for_crlf_patches() {
    let mut command = Command::new("printf");
    command.arg("a\\r\\nb\\r\\n");
    let (lines, _outcome) = collect_lines(&mut command, 1024).unwrap();
    assert_eq!(lines, vec!["a", "b"]);
}

#[test]
fn captures_nonzero_exit_status() {
    let mut command = Command::new("sh");
    command.args(["-c", "echo out; echo err 1>&2; exit 3"]);
    let (lines, outcome) = collect_lines(&mut command, 1024).unwrap();
    assert_eq!(lines, vec!["out"]);
    assert_eq!(outcome.status.code(), Some(3));
    assert_eq!(String::from_utf8_lossy(&outcome.stderr).trim(), "err");
}

// A ~10 MB single line proves chunked reassembly across many `CHUNK_BYTES`
// boundaries doesn't corrupt or split the line early, and that the bounded
// channel's backpressure doesn't deadlock a payload much larger than one
// chunk or one channel slot.
#[test]
fn handles_a_line_spanning_many_chunks_without_truncation() {
    const LINE_BYTES: usize = 10 * 1024 * 1024;
    // Generated via a pipeline (not passed as an argv string) since a 10 MB
    // command-line argument would exceed the OS's ARG_MAX.
    let mut command = Command::new("sh");
    command.args([
        "-c",
        &format!("head -c {LINE_BYTES} /dev/zero | tr '\\0' 'x'; echo"),
    ]);
    let (lines, outcome) = collect_lines(&mut command, LINE_BYTES + 16).unwrap();
    assert!(outcome.status.success());
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].len(), LINE_BYTES);
}

#[test]
fn rejects_a_line_exceeding_the_configured_cap() {
    let mut command = Command::new("printf");
    command.arg("x".repeat(200));
    let result = collect_lines(&mut command, 32);
    let error = result.expect_err("oversized line without a newline must error");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(
        error.to_string().contains("exceeds"),
        "unexpected error: {error}"
    );
}

#[test]
fn terminates_the_child_when_a_line_is_rejected() {
    // A long-running child whose output immediately exceeds the cap must be
    // killed rather than left running to completion in the background. The
    // child sleeps far longer than any plausible scheduling delay under a
    // loaded, fully-parallel test run, so a generous bound still only
    // passes if the process was actually terminated rather than awaited.
    let mut command = Command::new("sh");
    command.args(["-c", "printf '%0100d' 0; sleep 120"]);
    let start = std::time::Instant::now();
    let result = collect_lines(&mut command, 8);
    assert!(result.is_err());
    assert!(
        start.elapsed() < std::time::Duration::from_secs(60),
        "child should have been terminated well before its 120s sleep completed, \
         took {:?}",
        start.elapsed()
    );
}

#[test]
fn stderr_is_captured_even_when_stdout_is_empty() {
    let mut command = Command::new("sh");
    command.args(["-c", "echo only-err 1>&2"]);
    let (lines, outcome) = collect_lines(&mut command, 1024).unwrap();
    assert!(lines.is_empty());
    assert_eq!(String::from_utf8_lossy(&outcome.stderr).trim(), "only-err");
}

#[test]
fn stderr_beyond_the_cap_is_dropped_without_deadlocking() {
    // Emit far more than STDERR_CAP_BYTES to stderr while stdout stays
    // small; the call must still complete (the bounded reader keeps
    // draining past its cap instead of leaving the pipe full).
    let mut command = Command::new("sh");
    command.args(["-c", "yes err | head -c 200000 1>&2; echo done"]);
    let (lines, outcome) = collect_lines(&mut command, 1024).unwrap();
    assert_eq!(lines, vec!["done"]);
    assert!(outcome.status.success());
    assert!(outcome.stderr.len() <= STDERR_CAP_BYTES);
}

#[test]
fn respects_expired_invocation_deadline() {
    let _deadline = crate::invocation::install_test_deadline(std::time::Duration::ZERO).unwrap();
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 5"]);
    let result = collect_lines(&mut command, 1024);
    assert!(result.is_err());
}

#[test]
fn propagates_spawn_failure_for_a_missing_binary() {
    let mut command = Command::new("no-mistakes-definitely-not-a-real-binary");
    let result = collect_lines(&mut command, 1024);
    assert!(result.is_err());
}
