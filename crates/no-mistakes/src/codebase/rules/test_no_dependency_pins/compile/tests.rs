use super::{compile_builtin, compile_pattern};
use crate::codebase::rules::test_no_dependency_pins::RULE_ID;

#[test]
fn custom_patterns_report_everything_and_have_no_line_context() {
    let pattern = compile_pattern("custom", "(?<!@)pin-\\d+", false).unwrap();
    assert!(!pattern.builtin);
    assert!(pattern.reject_preceding_at);
    assert!(pattern.line_context.is_none());
    assert!(
        !compile_pattern("custom", "pin", true)
            .unwrap()
            .reject_preceding_at
    );
    assert!(compile_pattern("custom", "pin", true).unwrap().multiline);
}

#[test]
fn builtin_patterns_compile_their_line_context() {
    let plain = compile_builtin("builtin", "pin", false, None).unwrap();
    assert!(plain.builtin);
    assert!(plain.line_context.is_none());

    let contextual = compile_builtin("builtin", "pin", false, Some("brew")).unwrap();
    assert!(contextual.line_context.unwrap().is_match("brew install"));
}

#[test]
fn invalid_patterns_and_contexts_name_the_rule() {
    let error = compile_builtin("builtin", "[", false, None)
        .err()
        .expect("invalid pattern")
        .to_string();
    assert!(
        error.contains(RULE_ID) && error.contains("invalid pattern"),
        "{error}"
    );

    let error = compile_builtin("builtin", "pin", false, Some("["))
        .err()
        .expect("invalid line context")
        .to_string();
    assert!(
        error.contains(RULE_ID) && error.contains("invalid line context"),
        "{error}"
    );
}
