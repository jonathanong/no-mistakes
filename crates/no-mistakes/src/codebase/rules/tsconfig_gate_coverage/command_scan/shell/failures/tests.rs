use super::*;

#[test]
fn static_shell_failures_distinguish_successful_and_dynamic_forms() {
    for script in [
        "false",
        "true | false",
        "echo ok; exit 1",
        "false; exit",
        "false;\nexit",
        "return",
        "return; echo unreachable",
        "exit invalid",
        "exit 1 2",
    ] {
        assert!(shell_body_has_static_failure(script), "{script}");
    }
    for script in [
        "echo ok",
        "false | true",
        "exit",
        "true; exit",
        "true;\nexit",
        "exit 0",
        "exit 256",
        "false || true",
    ] {
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
        "false; exit",
    ] {
        assert!(
            shell_body_has_static_failure_with_initial(script, false),
            "{script}"
        );
    }
    for script in [
        "false; echo ok",
        "return; echo ok",
        "true; exit",
        "exit 0",
        "false || true",
    ] {
        assert!(
            !shell_body_has_static_failure_with_initial(script, false),
            "{script}"
        );
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
    assert!(shell_body_has_static_pipeline_failure(
        "tsc --noEmit --project app/tsconfig.json && false | true",
        true
    ));
    assert!(shell_body_has_static_pipeline_failure(
        "false | cat; tsc --noEmit --project app/tsconfig.json",
        true
    ));
    assert!(!shell_body_has_static_pipeline_failure(
        "set -e -o pipefail; false && false | true; echo after",
        true
    ));
}

#[test]
fn pipefail_failure_prefix_keeps_only_reachable_commands() {
    assert_eq!(
        shell_body_before_static_pipeline_failure(
            "tsc --noEmit --project before/tsconfig.json; false | true; tsc --noEmit --project after/tsconfig.json",
            true,
        ),
        "tsc --noEmit --project before/tsconfig.json"
    );
    assert_eq!(
        shell_body_before_static_pipeline_failure(
            "true && false | true; tsc --noEmit --project after/tsconfig.json",
            true,
        ),
        "true"
    );
    assert_eq!(
        shell_body_before_static_pipeline_failure("tsc --noEmit --project app/tsconfig.json", true),
        "tsc --noEmit --project app/tsconfig.json"
    );
}

#[test]
fn static_shell_success_requires_a_proven_successful_body() {
    for script in [
        "true",
        "true; exit",
        "true; exit 0",
        "true && true",
        "true | true",
        "false && true; true",
    ] {
        assert!(shell_body_is_statically_successful(script), "{script}");
    }
    for script in ["", "tsc --noEmit", "false"] {
        assert!(!shell_body_is_statically_successful(script), "{script}");
    }
}

#[test]
fn static_or_lists_remain_outside_the_supported_shell_subset() {
    for script in [
        "false || false",
        "false || true",
        "unknown || true",
        "true || false",
        "false | true || false",
    ] {
        assert!(!shell_body_has_static_failure(script), "{script}");
        assert!(!shell_body_is_statically_successful(script), "{script}");
    }
}
