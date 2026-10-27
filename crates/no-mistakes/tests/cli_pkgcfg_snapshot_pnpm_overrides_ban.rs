use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "pnpm-overrides-ban";

fn case(
    label: &'static str,
    scenario: &'static str,
    config: &'static str,
) -> (&'static str, PathBuf, PathBuf) {
    (
        label,
        rules_path(RULE, "fixture", scenario),
        rules_path(RULE, "configs", config),
    )
}

#[test]
fn pnpm_overrides_ban_json_matches_snapshot() {
    let cases = vec![
        case("pass", "pass", "repository.yml"),
        case("fail_workspace", "fail-workspace", "repository.yml"),
        case(
            "fail_package_overrides",
            "fail-package-overrides",
            "repository.yml",
        ),
        case(
            "fail_included_overrides",
            "fail-included-overrides",
            "repository.yml",
        ),
        case(
            "fail_included_selected",
            "fail-included-selected",
            "include-pnpm.yml",
        ),
        case(
            "fail_included_excluded",
            "fail-included-excluded",
            "include-workspace.yml",
        ),
        case(
            "fail_duplicate_overrides",
            "fail-duplicate-overrides",
            "repository.yml",
        ),
        case(
            "fail_pnpm_overrides",
            "fail-pnpm-overrides",
            "repository.yml",
        ),
        case("fail_yaml", "fail-yaml", "repository.yml"),
        case(
            "non_mapping_workspace",
            "non-mapping-workspace",
            "repository.yml",
        ),
        case("non_object_package", "non-object-package", "repository.yml"),
        case(
            "both_override_forms",
            "both-override-forms",
            "repository.yml",
        ),
    ];
    assert_saved_fixture("pnpm_overrides_ban", &cases);
}
