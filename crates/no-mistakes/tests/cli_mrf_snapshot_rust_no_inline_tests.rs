use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "rust-no-inline-tests";

#[test]
fn rust_no_inline_tests_json_matches_snapshot() {
    let repository = rules_path(RULE, "configs", "repository.yml");
    let mut cases = [
        ("clean", "unit", "clean.yml"),
        ("out_of_line", "unit", "out-of-line.yml"),
        ("simple_inline", "unit", "simple-inline.yml"),
        ("one_line", "unit", "one-line.yml"),
        ("pub_mod", "unit", "pub-mod.yml"),
        ("cfg_fn", "unit", "cfg-fn.yml"),
        ("cfg_use", "unit", "cfg-use.yml"),
        ("whitespace_cfg", "unit", "whitespace-cfg.yml"),
        ("extra_attrs", "unit", "extra-attrs.yml"),
        ("path_cfg", "unit", "path-cfg.yml"),
        ("disable_file", "unit", "disable-file.yml"),
        ("excludes", "respects-excludes", "excludes.yml"),
        ("relative_roots", "roots", "roots-sub.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(RULE, "fixture", fixture),
            rules_path(RULE, "configs", config),
        )
    })
    .to_vec();
    cases.extend([
        (
            "cfg_impl_items",
            rules_path(RULE, "fixture", "fail-associated"),
            repository.clone(),
        ),
        // readme.txt only: discovery finds no Rust file.
        (
            "no_rs_files",
            rules_path(RULE, "fixture", "no-rs"),
            repository.clone(),
        ),
        (
            "correct_path",
            rules_path(RULE, "fixture", "correct-path"),
            repository.clone(),
        ),
        (
            "sort_by_file",
            rules_path(RULE, "fixture", "sort-by-file"),
            repository,
        ),
        (
            "combined_invalid",
            rules_path("filesystem-dispatch", "rust-combined", "fixture"),
            rules_path(RULE, "configs", "combined-invalid.yml"),
        ),
    ]);
    assert_saved_fixture("rust_no_inline_tests", &cases);
}
