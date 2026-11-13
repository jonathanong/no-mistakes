use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

#[test]
fn postgres_grouping_review_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-grouping-not-aggregation/fixture"),
    );
    assert_saved_fixture(
        "postgres_grouping_review",
        &[
            (
                "guarded_absence_and_soundness_controls",
                root.clone(),
                root.join("review.yml"),
            ),
            (
                "existence_boundaries",
                root.join("existence-boundaries"),
                root.join("existence-boundaries/.no-mistakes.yml"),
            ),
        ],
    );
}
