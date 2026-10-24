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
        super::super::check_with_files(
            &root,
            &config,
            &[
                root.join(WORKFLOW),
                root.join(".github/workflows/compact.yml"),
                root.join(".github/actions/fetch/action.yml"),
            ],
        )
        .unwrap()
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

#[test]
fn trusted_command_alias_adds_uncertainty_without_changing_definite_findings() {
    let workflow = ".github/workflows/trusted-alias-subset.yml";
    let source = std::fs::read_to_string(fixture_root("effort-argv").join(workflow)).unwrap();
    let max = runs::check_source(workflow, &source, &hosts(), &[]);
    let low = runs::check_source_low(workflow, &source, &hosts(), &[], Unanalyzable::Report);
    assert_eq!(max.len(), 1, "{max:?}");
    assert_eq!(low.findings.len(), 2, "{:?}", low.findings);
    assert!(low.advisories.is_empty());
    let definite: Vec<_> = low
        .findings
        .iter()
        .filter(|finding| !finding.message.contains("unanalyzable at effort low"))
        .cloned()
        .collect();
    assert_eq!(definite, max);
    let shapes: Vec<_> = low
        .findings
        .iter()
        .filter(|finding| finding.message.contains("unanalyzable at effort low"))
        .collect();
    assert_eq!(shapes.len(), 1);
    assert!(shapes[0].message.contains("step 2 downloader command"));
    assert!(shapes[0].message.contains("call `curl` directly"));
    assert!(shapes[0]
        .message
        .contains("literal HTTP(S) URL arguments instead of a command alias"));
    assert!(max
        .iter()
        .all(|finding| !finding.message.contains("step 2 ")));

    // The URL is literal and trusted; only the command alias makes low uncertain.
    let parsed: serde_yaml::Value = serde_yaml::from_str(&source).unwrap();
    let script = parsed["jobs"]["verify"]["steps"][1]["run"]
        .as_str()
        .unwrap();
    let raw = judge::download_hits_low(script, &[]);
    assert_eq!(raw.len(), 1);
    assert!(raw[0].indirect_command && raw[0].unanalyzable);
    assert_eq!(raw[0].url, "https://api.github.com/repos/example/project");
}
