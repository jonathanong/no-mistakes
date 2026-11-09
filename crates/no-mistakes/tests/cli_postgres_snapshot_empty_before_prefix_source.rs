use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn empty_before_prefix_source_check_json_snapshot() {
    let fixture = "empty-before-prefix-source";
    let root = rules_path("postgres-require-query-annotation", "fixture", fixture);
    assert_saved_fixture(
        "postgres_annotation_empty_before_prefix_source",
        &[(fixture, root, Default::default())],
    );
}
