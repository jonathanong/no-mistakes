use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "markdown-mermaid-validation";

fn fixture_root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-mermaid-validation"),
    )
}

#[test]
fn markdown_mermaid_json_matches_snapshot() {
    assert_saved_fixture(
        "markdown_mermaid_validation",
        &[(
            "saved_fixtures",
            fixture_root(),
            rules_path(RULE, "configs", "selected-files.yml"),
        )],
    );
}
