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

// Regression for a review finding on #587: the terminated-line check used
// to compare `pos + 1` (the line plus its newline) against `max_line_bytes`,
// while the still-accumulating check compared the newline-free `pending.len()`
// — a one-byte inconsistency at the boundary. A line whose content is
// *exactly* `max_line_bytes` long must be accepted in both shapes, since
// `max_line_bytes` bounds the line "without a newline" per its own doc
// comment.
#[test]
fn accepts_a_line_exactly_at_the_cap_whether_or_not_it_is_newline_terminated() {
    let cap = 64;

    let mut terminated = Command::new("sh");
    terminated.args(["-c", &format!("printf '%{cap}d\\n' 0")]);
    let (lines, outcome) = collect_lines(&mut terminated, cap).unwrap();
    assert!(outcome.status.success());
    assert_eq!(lines[0].len(), cap);

    let mut unterminated = Command::new("printf");
    unterminated.arg("x".repeat(cap));
    let (lines, outcome) = collect_lines(&mut unterminated, cap).unwrap();
    assert!(outcome.status.success());
    assert_eq!(lines[0].len(), cap);
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

// Regression for a review finding on #587: the direct child can exit while
// a backgrounded descendant it spawned still holds the inherited stderr
// pipe open. Waiting for stderr must respect the invocation deadline (and
// terminate the whole process group, killing the orphaned descendant too)
// rather than joining unboundedly until that descendant's own pipe closes.
// The descendant closes its own inherited stdout (`1>&-`) before sleeping,
// so `drain_lines` (stdout) finishes normally and the stderr receive is the
// one step still pending — without that, the descendant would hold *both*
// pipes open and `drain_lines`'s own (already-covered) timeout would fire
// first, never exercising the stderr path this test targets. The deadline
// is generous (2s) so that spawn/drain_lines/wait_for_exit — each already
// deadline-checked separately — reliably complete well within budget under
// coverage instrumentation or CI load.
#[test]
fn stderr_wait_respects_the_deadline_when_a_descendant_holds_the_pipe_open() {
    let mut command = Command::new("sh");
    command.args(["-c", "(sleep 120 1>&-) & exit 0"]);
    let _deadline =
        crate::invocation::install_test_deadline(std::time::Duration::from_secs(2)).unwrap();
    let start = std::time::Instant::now();
    let result = collect_lines(&mut command, 1024);
    let error = result.expect_err("an orphaned descendant holding stderr open must time out");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(60),
        "should not have waited for the orphaned descendant's 120s sleep, took {:?}",
        start.elapsed()
    );
}

// Regression for a review finding on #587: the child must be terminated and
// reaped when the deadline elapses *while waiting for it to exit* (as
// opposed to already having elapsed before the wait, which the pre-spawn
// and pre-drain checks handle). The child closes stdout (so `drain_lines`
// completes successfully) but keeps running well past a short deadline.
#[test]
fn terminates_the_child_when_the_deadline_elapses_during_the_post_drain_wait() {
    let mut command = Command::new("sh");
    command.args(["-c", "printf 'x\\n'; exec 1>&-; sleep 120"]);
    let _deadline =
        crate::invocation::install_test_deadline(std::time::Duration::from_millis(200)).unwrap();
    let start = std::time::Instant::now();
    let result = collect_lines(&mut command, 1024);
    let error = result.expect_err("deadline elapsing during the post-drain wait must error");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(60),
        "child should have been terminated well before its 120s sleep completed, \
         took {:?}",
        start.elapsed()
    );
}

#[test]
fn respects_expired_invocation_deadline() {
    let _deadline = crate::invocation::install_test_deadline(std::time::Duration::ZERO).unwrap();
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 5"]);
    let result = collect_lines(&mut command, 1024);
    assert!(result.is_err());
}

// Distinct from `respects_expired_invocation_deadline`'s zero deadline
// (which expires before `drain_lines`'s first `remaining_timeout()` call,
// at the top of its loop): this deadline is still positive when `drain_lines`
// starts, so it reaches `rx.recv_timeout(remaining)` and must time out
// there, while blocked waiting for a chunk that never arrives.
#[test]
fn drain_lines_times_out_waiting_for_a_chunk() {
    let _deadline =
        crate::invocation::install_test_deadline(std::time::Duration::from_millis(200)).unwrap();
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 5"]);
    let start = std::time::Instant::now();
    let result = collect_lines(&mut command, 1024);
    let error = result.expect_err("deadline elapsing while waiting for a chunk must error");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(60),
        "child should have been terminated well before its 5s sleep completed, took {:?}",
        start.elapsed()
    );
}

// Regression for a review finding on #587: the deadline must be checked
// *before* spawning the child, matching `command_output`. Proven with a
// nonexistent binary — if the pre-spawn check were missing, `spawn()` would
// run first and fail with `NotFound`, not `TimedOut`.
#[test]
fn expired_deadline_prevents_child_spawn() {
    let _deadline = crate::invocation::install_test_deadline(std::time::Duration::ZERO).unwrap();
    let mut command = Command::new("definitely-not-a-real-no-mistakes-command");
    let error = collect_lines(&mut command, 1024).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
}

#[test]
fn propagates_spawn_failure_for_a_missing_binary() {
    let mut command = Command::new("no-mistakes-definitely-not-a-real-binary");
    let result = collect_lines(&mut command, 1024);
    assert!(result.is_err());
}
