use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "markdown-reachability";

fn saved_root(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/markdown-reachability")
            .join(name),
    )
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

#[test]
fn markdown_reachability_json_matches_snapshot() {
    let repository = config("repository-markdown.yml");
    let baseline = config("baseline.yml");
    let nested = config("nested-repository.yml");
    let cases = [
        (
            "direct_and_readme_paths",
            saved_root("paths"),
            repository.clone(),
        ),
        (
            "excess_depth",
            saved_root("paths"),
            config("paths-depth-one.yml"),
        ),
        (
            "invalid_intermediary",
            saved_root("invalid-intermediary"),
            repository.clone(),
        ),
        (
            "invalid_intermediary_baseline",
            saved_root("invalid-intermediary"),
            baseline.clone(),
        ),
        (
            "invalid_intermediary_index",
            saved_root("invalid-intermediary"),
            config("invalid-intermediary-index.yml"),
        ),
        (
            "reference_and_escaping_links",
            saved_root("links"),
            repository.clone(),
        ),
        (
            "baseline_exact",
            saved_root("baseline-match"),
            baseline.clone(),
        ),
        (
            "baseline_excluded_target",
            saved_root("baseline-match"),
            config("baseline-exclude-deep.yml"),
        ),
        (
            "baseline_root_stale",
            saved_root("baseline-root"),
            baseline.clone(),
        ),
        (
            "baseline_mismatched",
            saved_root("baseline-mismatched"),
            baseline,
        ),
        (
            "scoped_external_baseline",
            saved_root("scoped"),
            config("scoped-baseline.yml"),
        ),
        (
            "external_stale_path",
            saved_root("external-request"),
            config("external-stale.yml"),
        ),
        (
            "nested_project_scope",
            saved_root("nested-scope"),
            nested.clone(),
        ),
        (
            "scope_partition",
            saved_root("scope-partition"),
            nested.clone(),
        ),
        ("scope_alias", saved_root("scope-alias"), nested),
        (
            "scope_application_remapper",
            saved_root("scope-application-remapper"),
            config("scope-remapper.yml"),
        ),
        (
            "excluded_root_still_links",
            saved_root("filtered-target-graph"),
            config("filtered-guide.yml"),
        ),
        ("file_suppression", saved_root("suppression"), repository),
        (
            "three_hop_index_routes",
            saved_root("depth-three"),
            config("depth-three.yml"),
        ),
        (
            "two_hop_default_depth",
            saved_root("depth-three"),
            config("depth-three-default.yml"),
        ),
        (
            "two_hop_explicit_depth",
            saved_root("depth-three"),
            config("depth-three-depth-two.yml"),
        ),
        (
            "third_hop_baseline",
            saved_root("depth-three"),
            config("depth-three-baseline.yml"),
        ),
        (
            "shortcut_index_names",
            saved_root("depth-three"),
            config("depth-three-shortcut-index.yml"),
        ),
    ];
    assert_saved_fixture("markdown_reachability", &cases);
}
