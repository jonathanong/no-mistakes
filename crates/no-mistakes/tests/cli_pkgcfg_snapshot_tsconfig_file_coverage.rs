use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "tsconfig-file-coverage";

fn fixture(name: &str) -> PathBuf {
    rules_path(RULE, "fixture", name)
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

#[test]
fn tsconfig_file_coverage_json_matches_snapshot() {
    // Former discover_files + check_with_files output on directories whose
    // on-disk files are the inventory those tests passed. Explicit paths that
    // are not on disk, helper predicates, and the invalid include glob stay
    // in the unit tests.
    let empty = config("empty.yml");
    let pass = fixture("pass");
    let allow = fixture("allow");
    let auxiliary = fixture("auxiliary");
    assert_saved_fixture(
        "tsconfig_file_coverage",
        &[
            ("uncovered", fixture("fail"), empty.clone()),
            ("included", pass.clone(), empty.clone()),
            ("allow_orphan", allow.clone(), config("allow-generate.yml")),
            (
                "auxiliary_excluded",
                auxiliary.clone(),
                config("auxiliary.yml"),
            ),
            (
                "empty_allow_reason",
                allow,
                config("allow-empty-reason.yml"),
            ),
            ("stale_allow", pass.clone(), config("allow-stale.yml")),
            (
                "empty_auxiliary_reason",
                auxiliary.clone(),
                config("auxiliary-empty-reason.yml"),
            ),
            (
                "auxiliary_program_keys",
                pass.clone(),
                config("auxiliary-program-keys.yml"),
            ),
            (
                "auxiliary_basename",
                pass.clone(),
                config("auxiliary-basename.yml"),
            ),
            ("absolute_allow", pass.clone(), config("allow-absolute.yml")),
            (
                "windows_absolute_allow",
                pass.clone(),
                config("allow-windows.yml"),
            ),
            (
                "parent_traversal_allow",
                pass.clone(),
                config("allow-parent.yml"),
            ),
            ("empty_allow_path", pass, config("allow-empty-path.yml")),
            (
                "whitespace_required_basename",
                auxiliary.clone(),
                config("auxiliary-whitespace-basename.yml"),
            ),
            (
                "absolute_auxiliary",
                auxiliary,
                config("auxiliary-absolute.yml"),
            ),
            (
                "malformed_options",
                fixture("malformed-options"),
                config("malformed-options.yml"),
            ),
            (
                "source_include",
                fixture("source-include"),
                config("source-include.yml"),
            ),
            (
                "files_and_include",
                fixture("files-and-include"),
                empty.clone(),
            ),
            ("missing_tsconfig", fixture("missing-tsconfig"), empty),
            (
                "auxiliary_only",
                fixture("auxiliary-only"),
                config("auxiliary-only.yml"),
            ),
            (
                "auxiliary_array",
                fixture("auxiliary-array"),
                config("auxiliary-array.yml"),
            ),
            (
                "tracked_auxiliary",
                fixture("tracked-auxiliary"),
                config("tracked-auxiliary.yml"),
            ),
        ],
    );
}
