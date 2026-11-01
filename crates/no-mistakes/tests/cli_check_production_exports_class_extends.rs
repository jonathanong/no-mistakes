use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn production_exports_class_extends_json_matches_snapshot() {
    let production = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "class-extends-production",
    );
    let tests = rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "class-extends-tests",
    );
    assert_saved_fixture(
        "production_exports_class_extends",
        &[
            (
                "production",
                production.clone(),
                production.join(".no-mistakes.yml"),
            ),
            ("tests", tests.clone(), tests.join(".no-mistakes.yml")),
        ],
    );
}
