use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "workspace-package-cycles";

fn scenario(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(RULE)
            .join(name),
    )
}

fn case(
    label: &'static str,
    name: &'static str,
    config: &'static str,
) -> (&'static str, PathBuf, PathBuf) {
    (label, scenario(name), rules_path(RULE, "configs", config))
}

#[test]
fn workspace_package_cycles_json_matches_snapshot() {
    let cases = vec![
        case("cycle", "cycle", "default.yml"),
        case("allowlist", "cycle", "allowlist.yml"),
        case(
            "dev_dependencies_ignored",
            "dev-cycle",
            "dependencies-only.yml",
        ),
        // catalog:, unsatisfied ranges, and node_modules file: paths are not
        // workspace edges. See issue 2761.
        case("external_name", "external-name-collision", "default.yml"),
        case("workspace_self", "workspace-self-cycle", "default.yml"),
        case("star", "star-cycle", "default.yml"),
        case("range", "range-cycle", "default.yml"),
        case("file_path", "file-cycle", "default.yml"),
        case("link_path", "link-cycle", "default.yml"),
        case("portal_path", "portal-cycle", "default.yml"),
        case("unsatisfied_range", "unsatisfied-range", "default.yml"),
        case("non_string_version", "non-string-version", "default.yml"),
        case("workspace_path", "workspace-path-cycle", "default.yml"),
        case(
            "invalid_package_json",
            "invalid-package-json",
            "default.yml",
        ),
    ];
    assert_saved_fixture("workspace_package_cycles", &cases);
}
