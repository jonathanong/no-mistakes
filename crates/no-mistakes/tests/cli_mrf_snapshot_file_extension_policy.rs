use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "file-extension-policy";

#[test]
fn file_extension_policy_json_matches_snapshot() {
    let mut cases = [
        ("pass", "pass", "backend-js-ts.yml"),
        ("fail", "fail", "backend-js-ts.yml"),
        ("overlap", "overlap", "overlap.yml"),
        ("prefix_pass", "prefix-pass", "backend-js.yml"),
        (
            "banned_extension",
            "cases/banned-extension-in-scope-flagged",
            "backend-js-ts.yml",
        ),
        (
            "allowed_mts",
            "cases/allowed-extension-passes",
            "backend-js-ts.yml",
        ),
        (
            "declaration_dts",
            "cases/declaration-file-skipped",
            "backend-ts.yml",
        ),
        ("declarations_pass", "declarations-pass", "declarations.yml"),
        (
            "allowlisted",
            "cases/allowlisted-path-skipped",
            "allowlist.yml",
        ),
        (
            "out_of_scope",
            "cases/out-of-scope-path-not-flagged",
            "backend-js.yml",
        ),
        (
            "check_with_files",
            "cases/check-with-files-works",
            "backend-ts.yml",
        ),
        (
            "no_extension",
            "cases/file-with-no-extension-has-empty-ext-and-is-not-flagged",
            "backend-js-ts.yml",
        ),
        // `./src` is the dot-prefix fixture policy and one normalized prefix.
        ("dot_prefix", "dot-prefix-fail", "scope-dot-src.yml"),
        ("scope_src", "dot-prefix-fail", "scope-src.yml"),
        ("scope_dot_src", "dot-prefix-fail", "scope-dot-src.yml"),
        (
            "scope_src_parent_src",
            "dot-prefix-fail",
            "scope-src-parent-src.yml",
        ),
        ("scope_src_slash", "dot-prefix-fail", "scope-src-slash.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(RULE, "fixture", fixture),
            rules_path(RULE, "configs", config),
        )
    })
    .to_vec();
    // Escape findings name the loaded config. Keep that file inside the
    // fixture so the report path stays repository-relative.
    cases.extend([
        (
            "scope_escape",
            rules_path(RULE, "fixture", "scope-escape"),
            rules_path(RULE, "fixture", "scope-escape/.no-mistakes.yml"),
        ),
        (
            "escape_parent",
            rules_path(RULE, "fixture", "dot-prefix-fail"),
            rules_path(RULE, "fixture", "dot-prefix-fail/scope-parent.yml"),
        ),
        (
            "escape_parent_src",
            rules_path(RULE, "fixture", "dot-prefix-fail"),
            rules_path(RULE, "fixture", "dot-prefix-fail/scope-parent-src.yml"),
        ),
        (
            "escape_src_parent_parent",
            rules_path(RULE, "fixture", "dot-prefix-fail"),
            rules_path(
                RULE,
                "fixture",
                "dot-prefix-fail/scope-src-parent-parent.yml",
            ),
        ),
        (
            "escape_foo_parent_src",
            rules_path(RULE, "fixture", "dot-prefix-fail"),
            rules_path(RULE, "fixture", "dot-prefix-fail/scope-foo-escape.yml"),
        ),
    ]);
    assert_saved_fixture("file_extension_policy", &cases);
}
