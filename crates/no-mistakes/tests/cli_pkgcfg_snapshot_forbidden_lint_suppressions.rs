use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn cfg(name: &str) -> PathBuf {
    rules_path("forbidden-lint-suppressions", "configs", name)
}

#[test]
fn forbidden_lint_suppressions_json_matches_snapshot() {
    let root = rules_path("forbidden-lint-suppressions", "fixture", ".");
    assert_saved_fixture(
        "forbidden_lint_suppressions",
        &[
            ("default", root.clone(), cfg("default.yml")),
            ("explicit", root.clone(), cfg("explicit.yml")),
            ("blanket", root.clone(), cfg("blanket.yml")),
            ("typescript", root.clone(), cfg("ts.yml")),
            ("no_directives", root.clone(), cfg("none.yml")),
            ("no_matches", root.clone(), cfg("no-matches.yml")),
            ("excluded", root.clone(), cfg("excluded.yml")),
            ("custom_message", root.clone(), cfg("custom.yml")),
            ("quoted", root.clone(), cfg("quoted.yml")),
            ("html_comments", root, cfg("html.yml")),
        ],
    );
}
