use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "vite-optimize-deps-nested-imports";

fn case(label: &'static str, scenario: &'static str) -> (&'static str, PathBuf, PathBuf) {
    (
        label,
        rules_path(RULE, "fixture", scenario),
        rules_path(RULE, "fixture", scenario).join(".no-mistakes.yml"),
    )
}

#[test]
fn vite_optimize_deps_nested_imports_json_matches_snapshot() {
    let cases = vec![
        case("reported", "reported"),
        case("covered", "covered"),
        case("builtins", "builtins"),
        case("unanalyzable_configs", "unanalyzable-configs"),
        case("manifest_gaps", "manifest-gaps"),
        case("missing_config", "missing-config"),
        case("empty_configs", "empty-configs"),
        case("empty_packages", "empty-packages"),
        case("omitted_both", "omitted-both"),
        case("omitted_configs", "omitted-configs"),
        case("omitted_packages", "omitted-packages"),
        case("config_edges", "config-edges"),
        case("manifest_edges", "manifest-edges"),
        case("barrel_entries", "barrel-entries"),
        case("include_sources", "include-sources"),
        case("include_source_gap", "include-source-gap"),
        case("workspace_children", "workspace-children"),
        case("ignored_packages", "ignored-packages"),
        case("bundle_flags", "bundle-flags"),
        case("declared_builtins", "declared-builtins"),
        case("asset_imports", "asset-imports"),
        case("nested_conditions", "nested-conditions"),
        case("duplicate_names", "duplicate-names"),
        case("include_chains", "include-chains"),
        case("top_level_conditions", "top-level-conditions"),
        case("exported_styles", "exported-styles"),
        case("const_includes", "const-includes"),
        case("specifier_subpaths", "specifier-subpaths"),
    ];
    assert_saved_fixture("vite_optimize_deps_nested_imports", &cases);
}
