use super::*;

#[test]
fn static_shell_failures_distinguish_successful_and_dynamic_forms() {
    for script in [
        "false",
        "echo ok; exit 1",
        "return",
        "exit invalid",
        "exit 1 2",
    ] {
        assert!(shell_body_has_static_failure(script), "{script}");
    }
    for script in ["echo ok", "exit", "exit 0", "exit 256", "false || true"] {
        assert!(!shell_body_has_static_failure(script), "{script}");
    }
    assert!(!shell_body_has_static_failure(
        "false && echo masked; echo completed"
    ));
}

#[test]
fn terminal_failures_ignore_masked_non_errexit_commands() {
    for script in [
        "exit 1",
        "echo ok; exit 2",
        "false",
        "false && echo skipped",
    ] {
        assert!(shell_body_has_static_terminal_failure(script), "{script}");
    }
    for script in ["false; echo ok", "exit 0", "false || true"] {
        assert!(!shell_body_has_static_terminal_failure(script), "{script}");
    }
}

#[test]
fn pipefail_detects_static_pipeline_failures_without_and_or_masking() {
    assert!(shell_body_has_static_pipeline_failure(
        "false | true; echo unreachable",
        true
    ));
    for script in [
        "false | true",
        "false | true && echo masked",
        "true && false | true",
        "true | true && false | true",
    ] {
        assert!(
            shell_body_has_static_pipeline_failure(script, false),
            "{script}"
        );
    }
    for script in [
        "true | true",
        "false | true || echo recovered",
        "false | true && echo masked; echo completed",
    ] {
        assert!(
            !shell_body_has_static_pipeline_failure(script, true),
            "{script}"
        );
    }
}
