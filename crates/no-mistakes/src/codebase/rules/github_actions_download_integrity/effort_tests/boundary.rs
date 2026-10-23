use super::*;

#[test]
fn standalone_default_max_matches_the_prepared_rule_boundary() {
    let root = fixture_root("fail");
    let config = NoMistakesConfig {
        rules: vec![rule(Effort::Max, Unanalyzable::Report)],
        ..NoMistakesConfig::default()
    };
    let standalone = super::super::check(&root, &config).unwrap();
    assert!(!standalone.is_empty());
    assert_eq!(
        standalone,
        super::super::check_with_files(&root, &config, &[root.join(WORKFLOW)]).unwrap()
    );
}

#[test]
fn low_ignores_non_actions_malformed_yaml_and_unreadable_inventory_entries() {
    let root = fixture_root("effort-argv");
    let malformed =
        std::fs::read_to_string(root.join(".github/workflows/malformed-low.yml")).unwrap();
    for (path, content) in [("README.md", source("fail")), (WORKFLOW, malformed)] {
        let result = runs::check_source_low(path, &content, &hosts(), &[], Unanalyzable::Report);
        assert!(result.findings.is_empty());
        assert!(result.advisories.is_empty());
    }
    // Missing inventory paths model a file removed after discovery; do not create it.
    let files = vec![root.join(".github/workflows/removed-after-discovery.yml")];
    assert!(!files[0].exists());
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
    let config = NoMistakesConfig {
        rules: vec![rule(Effort::Low, Unanalyzable::Report)],
        ..NoMistakesConfig::default()
    };
    let result =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert!(result.findings.is_empty());
    assert!(result.advisories.is_empty());
}

#[test]
fn command_alias_diagnostics_name_the_direct_call_rewrite() {
    let result = runs::check_source_low(
        WORKFLOW,
        &source("effort-argv"),
        &hosts(),
        &[],
        Unanalyzable::Advise,
    );
    assert!(result.advisories.iter().any(|row| row
        .message
        .contains("downloader command is unanalyzable at effort low")
        && row.message.contains("directly")));
}
