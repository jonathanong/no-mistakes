use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "tsconfig-alias-folder-mapping";

fn fixture(name: &str) -> PathBuf {
    rules_path(RULE, "fixture", name)
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

#[test]
fn tsconfig_alias_folder_mapping_json_matches_snapshot() {
    // Former check / check_with_files output. Two cases stay in the unit tests:
    // a nonexistent tsconfig paired with include `[` (config load rejects the
    // glob first), and extends-missing (the CLI exits 2 with no JSON when the
    // extended file is absent).
    let agents = config("agents.yml");
    let locations = fixture("locations");
    assert_saved_fixture(
        "tsconfig_alias_folder_mapping",
        &[
            ("directory_mappings_pass", fixture("pass"), agents.clone()),
            ("valid_files", fixture("valid-files"), agents.clone()),
            ("wrong_files", fixture("wrong-files"), agents.clone()),
            ("fail", fixture("fail"), agents.clone()),
            ("no_options", fixture("fail"), config("empty-options.yml")),
            (
                "multiple_targets",
                fixture("multiple-targets"),
                agents.clone(),
            ),
            ("no_paths", fixture("no-paths"), agents.clone()),
            ("wrong_prefix", fixture("wrong-prefix"), agents.clone()),
            (
                "two_mappings",
                fixture("two-mappings"),
                config("two-mappings.yml"),
            ),
            (
                "exists_pass",
                fixture("exists-pass"),
                config("check-exists.yml"),
            ),
            (
                "exists_fail",
                fixture("exists-fail"),
                config("check-exists.yml"),
            ),
            (
                "exists_shape_without_check",
                fixture("exists-fail"),
                agents.clone(),
            ),
            (
                "empty_base",
                fixture("empty-base"),
                config("empty-base.yml"),
            ),
            (
                "shared_root",
                fixture("shared-root"),
                config("shared-root.yml"),
            ),
            (
                "valid_files_second_mapping",
                fixture("valid-files"),
                config("agents-and-other.yml"),
            ),
            ("locations", locations.clone(), agents),
            (
                "locations_empty_paths",
                locations.clone(),
                config("tsconfig-empty.yml"),
            ),
            (
                "locations_array_empty",
                locations.clone(),
                config("tsconfig-array-empty.yml"),
            ),
            (
                "locations_child",
                locations.clone(),
                config("tsconfig-child.yml"),
            ),
            (
                "locations_include_tsconfig",
                locations.clone(),
                config("include-tsconfig.yml"),
            ),
            (
                "locations_include_config",
                locations.clone(),
                config("include-config-json.yml"),
            ),
            (
                "locations_exclude_tsconfig",
                locations,
                config("exclude-tsconfig.yml"),
            ),
        ],
    );
}
