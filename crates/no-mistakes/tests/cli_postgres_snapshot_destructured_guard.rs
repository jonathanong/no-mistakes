use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "postgres-require-query-annotation";

#[test]
fn postgres_destructured_guarded_return_check_json_snapshot() {
    let cases = [(
        "destructured-guarded-return",
        rules_path(RULE, "fixture", "destructured-guarded-return"),
        rules_path(RULE, "configs", "destructured-guarded-return.yml"),
    )];
    let document =
        assert_saved_fixture_report("postgres_destructured_guarded_return", &cases, &[], &[]);
    let findings = document["destructured-guarded-return"]["report"]["rules"]
        .as_array()
        .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn postgres_destructured_wrapper_shapes_check_json_snapshot() {
    let cases = [(
        "destructured-wrapper-shapes",
        rules_path(RULE, "fixture", "destructured-wrapper-shapes"),
        rules_path(RULE, "configs", "destructured-wrapper-shapes.yml"),
    )];
    let document =
        assert_saved_fixture_report("postgres_destructured_wrapper_shapes", &cases, &[], &[]);
    let findings = document["destructured-wrapper-shapes"]["report"]["rules"]
        .as_array()
        .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn postgres_destructured_guard_writes_check_json_snapshot() {
    let cases = [(
        "destructured-guard-writes",
        rules_path(RULE, "fixture", "destructured-guard-writes"),
        rules_path(RULE, "configs", "destructured-guard-writes.yml"),
    )];
    assert_saved_fixture_report("postgres_destructured_guard_writes", &cases, &[], &[]);
}

#[test]
fn postgres_destructured_shape_parity_check_json_snapshot() {
    let cases = [(
        "destructured-shape-parity",
        rules_path(RULE, "fixture", "destructured-shape-parity"),
        rules_path(RULE, "configs", "destructured-shape-parity.yml"),
    )];
    assert_saved_fixture_report("postgres_destructured_shape_parity", &cases, &[], &[]);
}
