use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "no-sparse-checkout";

fn review_root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/repository-guard-review/no-sparse-checkout"),
    )
}

#[test]
fn no_sparse_checkout_json_matches_snapshot() {
    let empty = rules_path(RULE, "configs", "empty.yml");
    assert_saved_fixture(
        "no_sparse_checkout",
        &[
            (
                "workflows",
                rules_path(RULE, "fixture", "fail"),
                empty.clone(),
            ),
            (
                "custom_include",
                rules_path(RULE, "fixture", "custom"),
                rules_path(RULE, "configs", "custom-include.yml"),
            ),
            (
                "default_include_miss",
                rules_path(RULE, "fixture", "custom"),
                empty.clone(),
            ),
            (
                "composite_action",
                rules_path(RULE, "fixture", "action"),
                empty.clone(),
            ),
            // Post-suppression only: continuations.yml is fully suppressed.
            ("review_locations", review_root(), empty),
        ],
    );
}
