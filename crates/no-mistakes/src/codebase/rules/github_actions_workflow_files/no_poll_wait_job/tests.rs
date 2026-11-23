use super::loops::poll_loops;

fn polls() -> Vec<Vec<String>> {
    vec![vec!["gh".into(), "run".into(), "view".into()]]
}

fn lines(script: &str, skip_for: bool) -> Vec<usize> {
    poll_loops(script, &polls(), skip_for)
        .into_iter()
        .map(|hit| hit.line)
        .collect()
}

#[test]
fn until_condition_poll_with_sleep_body_is_reported() {
    let script = "until gh run view 123 --json status; do sleep 30; done";
    assert_eq!(lines(script, false), vec![1]);
}

#[test]
fn command_substitution_poll_in_a_while_loop_is_reported() {
    let script = "echo start\nwhile [ \"$(gh run view 1 -q .status)\" != completed ]; do\n  sleep 5\ndone";
    assert_eq!(lines(script, false), vec![2]);
}

#[test]
fn body_poll_in_a_for_loop_is_reported_unless_for_loops_are_ignored() {
    let script = "for i in 1 2 3; do\n  gh run view 1 || true\n  sleep 5\ndone";
    assert_eq!(lines(script, false), vec![1]);
    assert!(lines(script, true).is_empty());
}

#[test]
fn nested_loop_reports_each_loop_that_holds_both() {
    let script = "while true; do\n  until gh run view 1; do sleep 1; done\ndone";
    assert_eq!(lines(script, false), vec![2]);
}

#[test]
fn abstains_without_a_sleep_a_poll_or_a_loop() {
    assert!(lines("sleep 30\ngh run view 1", false).is_empty());
    assert!(lines("while true; do sleep 1; done", false).is_empty());
    assert!(lines("until gh run view 1; do echo wait; done", false).is_empty());
    assert!(lines("echo gh run view 1; sleep 3", false).is_empty());
}

#[test]
fn abstains_on_unbalanced_scripts_and_heredocs() {
    assert!(lines("until gh run view 1; do sleep 1", false).is_empty());
    assert!(lines("done\nuntil gh run view 1; do sleep 1; done", false).is_empty());
    let heredoc = "cat <<EOF\nuntil gh run view 1; do sleep 1; done\nEOF";
    assert!(lines(heredoc, false).is_empty());
}

#[test]
fn unconfigured_commands_are_not_reported() {
    let script = "until curl -fs https://example.invalid/ready; do sleep 2; done";
    assert!(lines(script, false).is_empty());
}
