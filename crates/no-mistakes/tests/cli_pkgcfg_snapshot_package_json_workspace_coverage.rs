use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "package-json-workspace-coverage";

fn root(name: &str) -> PathBuf {
    rules_path(RULE, name, ".")
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

#[test]
fn package_json_workspace_coverage_json_matches_snapshot() {
    // Includes on `missing` repeat the old explicit file lists. Other
    // package.json files in that directory change coverage when selected.
    let services = config("packages-and-services.yml");
    let require_named = config("require-named.yml");
    assert_saved_fixture(
        "package_json_workspace_coverage",
        &[
            (
                "missing_roots",
                root("missing"),
                config("missing-selected.yml"),
            ),
            ("allowlist", root("missing"), config("allowlist.yml")),
            (
                "allowlist_relative",
                root("missing"),
                config("allowlist-relative.yml"),
            ),
            (
                "unnamed_selected",
                root("missing"),
                config("unnamed-selected.yml"),
            ),
            (
                "malformed_named",
                root("invalid-package-json"),
                require_named.clone(),
            ),
            (
                "star_does_not_cross",
                root("unnamed-covered"),
                require_named,
            ),
            (
                "names_not_required",
                root("unnamed-covered"),
                config("package-roots.yml"),
            ),
            (
                "unmatched_pattern",
                root("unmatched-pattern"),
                config("packages-and-apps.yml"),
            ),
            (
                "unmatched_package_json",
                root("unmatched-package-json"),
                config("package-roots.yml"),
            ),
            (
                "included_package_globs",
                root("included-packages"),
                services.clone(),
            ),
            (
                "duplicate_packages",
                root("duplicate-packages"),
                services.clone(),
            ),
            (
                "duplicate_narrowed",
                root("duplicate-packages"),
                config("duplicate-narrow.yml"),
            ),
            (
                "delegated_omitted",
                root("delegated-omitted"),
                services.clone(),
            ),
            ("unreadable_packages", root("unreadable-packages"), services),
            (
                "scoped_present",
                root("scoped-app"),
                config("scoped-present.yml"),
            ),
            (
                "scoped_packages_only",
                root("scoped-app"),
                config("scoped-packages-only.yml"),
            ),
        ],
    );
}
