use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "package-json-required-fields";

#[test]
fn package_json_required_fields_json_matches_snapshot() {
    let shape = rules_path(RULE, "configs", "shape.yml");
    let empty = rules_path(RULE, "configs", "empty.yml");
    assert_saved_fixture(
        "package_json_required_fields",
        &[
            ("shape", rules_path(RULE, "fixture", "fail"), shape.clone()),
            ("pass", rules_path(RULE, "fixture", "pass"), shape.clone()),
            (
                "unscoped_exception",
                rules_path(RULE, "fixture", "exception"),
                shape,
            ),
            (
                "empty_options",
                rules_path(RULE, "fixture", "fail"),
                empty.clone(),
            ),
            (
                "empty_options_unanalyzable",
                rules_path(RULE, "fixture", "unanalyzable"),
                empty,
            ),
            (
                "main_companion_included",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "main-index-mts.yml"),
            ),
            (
                "include_valid_only",
                rules_path(RULE, "fixture", "unanalyzable"),
                rules_path(RULE, "configs", "include-valid.yml"),
            ),
            (
                "nested_companion",
                rules_path(RULE, "fixture", "nested-companion"),
                rules_path(RULE, "configs", "main-src-index.yml"),
            ),
        ],
    );
}
