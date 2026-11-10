use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn production_exports_issue_3221_check_json_snapshot() {
    let mut rows = Vec::new();
    for case in ["unreachable", "imported", "inside-roots", "unknown-closure"] {
        let root = rules_path(
            "production-exports-only-used-by-tests",
            "fixture",
            &format!("issue-3221/{case}"),
        );
        rows.push((case, root.clone(), root.join(".no-mistakes.yml")));
    }
    assert_saved_fixture("production_exports_issue_3221", &rows);
}
