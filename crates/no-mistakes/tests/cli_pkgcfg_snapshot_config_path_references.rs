use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "config-path-references";

fn root(kind: &str, name: &str) -> PathBuf {
    rules_path(RULE, kind, name)
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

#[test]
fn config_path_references_json_matches_snapshot() {
    // Former check_with_files output whose file list is what CLI discovery
    // returns for that directory. Preset configs keep the projects, tests, and
    // testPlan sections the no-mistakes preset reads from the loaded config.
    // Knip fixtures under a `fixtures` directory stay in the unit tests.
    let fixture = root(".", "fixture");
    assert_saved_fixture(
        "config_path_references",
        &[
            (
                "missing_literal",
                fixture.clone(),
                config("missing-literal.yml"),
            ),
            (
                "glob_outside_include",
                fixture.clone(),
                config("glob-outside-include.yml"),
            ),
            ("invalid_yaml", fixture, config("unreadable-config.yml")),
            (
                "presets_fail",
                root(".", "presets-fail"),
                config("presets-fail.yml"),
            ),
            (
                "presets_pass",
                root(".", "presets-pass"),
                config("presets-pass.yml"),
            ),
            (
                "invalid_sgconfig",
                root("fixture", "cases/invalid-sgconfig"),
                config("sgconfig.yml"),
            ),
            (
                "disable_file",
                root("suppression", "disable-file"),
                config("sgconfig.yml"),
            ),
            (
                "disable_line",
                root("suppression", "disable-line"),
                config("sgconfig.yml"),
            ),
            // The preset anchors missing paths at line 1, above the directive target.
            (
                "first_line_next_line_directive",
                root("suppression", "disable-next-line"),
                config("sgconfig.yml"),
            ),
            (
                "pnpm_invalid_yaml",
                root("fixture", "cases/pnpm-filter-invalid-yaml"),
                config("pnpm-invalid-yaml.yml"),
            ),
        ],
    );
}
