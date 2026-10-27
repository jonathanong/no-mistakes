use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "banned-renamed-files";

#[test]
fn banned_renamed_files_json_matches_snapshot() {
    let web = rules_path(RULE, "configs", "middleware-web.yml");
    let any = rules_path(RULE, "configs", "middleware-any.yml");
    assert_saved_fixture(
        "banned_renamed_files",
        &[
            (
                "non_banned_name",
                rules_path(RULE, "fixture", "cases/non-banned-name-passes"),
                rules_path(RULE, "configs", "middleware-web.yml"),
            ),
            (
                "compound_basename",
                rules_path(RULE, "fixture", "cases/compound-flags-single-and-multipart"),
                rules_path(RULE, "configs", "compound-any.yml"),
            ),
            (
                "compound_unrelated_prefix",
                rules_path(
                    RULE,
                    "fixture",
                    "cases/compound-does-not-match-unrelated-prefix",
                ),
                rules_path(RULE, "configs", "compound-any.yml"),
            ),
            (
                "regex_basename",
                rules_path(
                    RULE,
                    "fixture",
                    "cases/regex-matches-basename-ignoring-extensions-filter",
                ),
                rules_path(RULE, "configs", "regex-any.yml"),
            ),
            (
                "regex_non_match",
                rules_path(RULE, "fixture", "cases/regex-non-match-passes"),
                rules_path(RULE, "configs", "regex-any.yml"),
            ),
            ("pass", rules_path(RULE, "fixture", "pass"), web.clone()),
            (
                "in_scope",
                rules_path(RULE, "fixture", "cases/banned-basename-in-scope-flagged"),
                web.clone(),
            ),
            (
                "fail_suppressed",
                rules_path(RULE, "fixture", "fail"),
                web.clone(),
            ),
            (
                "root_scope",
                rules_path(RULE, "fixture", "root-scope-fail"),
                rules_path(RULE, "configs", "middleware-dot-scope.yml"),
            ),
            (
                "prefix_scope",
                rules_path(RULE, "fixture", "prefix-pass"),
                web.clone(),
            ),
            (
                "out_of_scope",
                rules_path(RULE, "fixture", "cases/out-of-scope-path-not-flagged"),
                web.clone(),
            ),
            (
                "non_matching_extension",
                rules_path(RULE, "fixture", "cases/non-matching-extension-not-flagged"),
                web,
            ),
            (
                "compound",
                rules_path(RULE, "fixture", "compound-fail"),
                rules_path(RULE, "configs", "compound-web.yml"),
            ),
            (
                "compound_extension",
                rules_path(RULE, "fixture", "cases/compound-still-respects-extensions"),
                rules_path(RULE, "configs", "compound-any.yml"),
            ),
            (
                "regex_ignores_extensions",
                rules_path(RULE, "fixture", "regex-fail"),
                rules_path(RULE, "configs", "regex-narrow.yml"),
            ),
            (
                "no_scope",
                rules_path(RULE, "fixture", "cases/no-scope-matches-all-paths"),
                any.clone(),
            ),
            (
                "no_dot_extension",
                rules_path(
                    RULE,
                    "fixture",
                    "cases/file-with-no-dot-not-flagged-for-dot-extension",
                ),
                any,
            ),
            (
                "mts_extension",
                rules_path(RULE, "fixture", "cases/check-with-files-works"),
                rules_path(RULE, "configs", "middleware-mts.yml"),
            ),
        ],
    );
}
