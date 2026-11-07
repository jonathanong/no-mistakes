use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn low_copy_pass_through_check_json_snapshot() {
    shape(
        "low-copy-pass-through",
        "postgres_annotation_low_copy_pass_through",
    );
}

#[test]
fn low_conditional_callee_check_json_snapshot() {
    shape(
        "low-conditional-callee",
        "postgres_annotation_low_conditional_callee",
    );
}

fn shape(fixture: &str, snapshot: &str) {
    let root = rules_path("postgres-require-query-annotation", "fixture", fixture);
    assert_saved_fixture(snapshot, &[(fixture, root, Default::default())]);
}
