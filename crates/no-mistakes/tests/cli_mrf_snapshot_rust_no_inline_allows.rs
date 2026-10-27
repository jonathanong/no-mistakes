use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "rust-no-inline-allows";

#[test]
fn rust_no_inline_allows_json_matches_snapshot() {
    let mut cases = [
        ("clean", "unit", "clean.yml"),
        ("inline_allow", "unit", "inline-allow.yml"),
        ("indented_allow", "unit", "indented-allow.yml"),
        ("path_allow", "unit", "path-allow.yml"),
        ("namespaced_allow", "unit", "namespaced-allow.yml"),
        ("invalid", "unit", "invalid.yml"),
        ("disable_file", "unit", "disable-file.yml"),
        ("excludes", "excludes", "excludes.yml"),
        ("relative_roots", "roots", "roots-sub.yml"),
        // roots/tests.rs has an allow attribute and must stay out of the report.
        ("skips_test_files", "roots", "repository.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(RULE, "fixture", fixture),
            rules_path(RULE, "configs", config),
        )
    })
    .to_vec();
    cases.push((
        "combined_invalid",
        rules_path("filesystem-dispatch", "rust-combined", "fixture"),
        rules_path(RULE, "configs", "combined-invalid.yml"),
    ));
    assert_saved_fixture("rust_no_inline_allows", &cases);
}
