use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "package-json-nested-workspace-coverage";

fn root(name: &str) -> PathBuf {
    rules_path(RULE, name, ".")
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

#[test]
fn package_json_nested_workspace_coverage_json_matches_snapshot() {
    let shared = config("shared.yml");
    assert_saved_fixture(
        "package_json_nested_workspace_coverage",
        &[
            ("valid", root("valid"), shared.clone()),
            ("parent_path", root("parent-path"), shared.clone()),
            (
                "sibling_prefix",
                root("sibling-prefix"),
                config("sibling-prefix.yml"),
            ),
            ("extra", root("extra"), shared.clone()),
            ("missing", root("missing"), shared.clone()),
            ("glob", root("glob"), shared.clone()),
            ("brace_glob", root("brace-glob"), shared.clone()),
            ("lambda", root("lambda"), shared.clone()),
            ("partial", root("partial"), config("partial.yml")),
            ("unresolved", root("unresolved"), shared.clone()),
            ("empty_roots", root("valid"), config("empty-roots.yml")),
            (
                "empty_prefixes",
                root("valid"),
                config("empty-prefixes.yml"),
            ),
        ],
    );
}
