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
}
