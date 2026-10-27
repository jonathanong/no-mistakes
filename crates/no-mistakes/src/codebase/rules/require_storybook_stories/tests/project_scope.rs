use super::*;

// Output cases moved to cli_rra_snapshot_require_storybook_stories.

fn scope_config(name: &str) -> NoMistakesConfig {
    let root = fixture("project-include-scope");
    crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap()
}

#[test]
fn multiple_project_targets_keep_the_library_diagnostic() {
    let root = fixture("project-include-scope");
    let config = "invalid-multiple-targets.yml";
    let direct = check(&root, &scope_config(config), None)
        .unwrap_err()
        .to_string();
    let aggregate = crate::codebase::rules::run_check(&root, Some(&root.join(config)), None)
        .unwrap_err()
        .to_string();
    assert!(direct.contains("requires exactly one project target"));
    assert_eq!(direct, aggregate);
}

#[test]
fn project_scope_direct_matches_aggregate() {
    let root = fixture("project-include-scope");
    for config in [
        "generic-include.yml",
        "overlapping-projects.yml",
        "exclude-repository-relative.yml",
        "exclude-project-relative.yml",
        "options-include.yml",
        "additive-options-include.yml",
    ] {
        let direct = check(&root, &scope_config(config), None).unwrap();
        let aggregate =
            crate::codebase::rules::run_check(&root, Some(&root.join(config)), None).unwrap();
        assert_eq!(direct, aggregate, "{config}");
    }
}
