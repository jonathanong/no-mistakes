use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "postgres-require-query-annotation";

fn assert_reports(name: &str, fixture: &str) {
    let cases = [(
        fixture,
        rules_path(RULE, "fixture", fixture),
        rules_path(RULE, "configs", "soundness-negative.yml"),
    )];
    let document = assert_saved_fixture_report(name, &cases, &[], &[]);
    let findings = document[fixture]["report"]["rules"].as_array().unwrap();
    assert!(!findings.is_empty(), "{fixture} must still report");
}

#[test]
fn postgres_soundness_unsupported_caller_check_json_snapshot() {
    assert_reports(
        "postgres_soundness_unsupported_caller",
        "soundness-unsupported-caller",
    );
}

#[test]
fn postgres_soundness_dynamic_external_caller_check_json_snapshot() {
    assert_reports(
        "postgres_soundness_dynamic_external_caller",
        "soundness-dynamic-external-caller",
    );
}

#[test]
fn postgres_soundness_guard_mutates_builder_check_json_snapshot() {
    assert_reports(
        "postgres_soundness_guard_mutates_builder",
        "soundness-guard-mutates-builder",
    );
}
