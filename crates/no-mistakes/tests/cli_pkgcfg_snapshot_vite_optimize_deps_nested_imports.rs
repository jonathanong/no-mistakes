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
    ];
    assert_saved_fixture("vite_optimize_deps_nested_imports", &cases);
}
