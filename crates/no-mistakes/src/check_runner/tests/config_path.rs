use super::*;

#[test]
fn empty_results_records_cli_side_channels() {
    let results = results::empty_results([Some("warning".to_string())]);
    assert!(!results.warnings.is_empty());
    assert!(!results.timings.is_empty());
    assert!(results.react.is_empty());
    assert!(results.queues.is_empty());
    assert!(results.rules.is_empty());
    assert!(results.integration.is_empty());
    assert!(results.codebase.is_empty());
}

#[test]
fn run_all_returns_empty_results_when_no_check_domain_is_configured() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-runner/empty");

    let results = run_all(root, None, None).unwrap();

    assert!(results.react.is_empty());
    assert!(results.queues.is_empty());
    assert!(results.rules.is_empty());
    assert!(results.integration.is_empty());
    assert!(results.codebase.is_empty());
}

#[test]
fn auto_discovered_config_path_reaches_findings_and_suppressions() {
    let fixture_root = |name: &str| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/tsconfig-gate-coverage")
            .join(name)
    };
    let results = run_all(fixture_root("auto-config-path"), None, None).unwrap();
    assert_eq!(results.rules.len(), 1, "{:?}", results.rules);
    assert_eq!(results.rules[0].rule, "tsconfig-gate-coverage");
    assert_eq!(results.rules[0].file, ".no-mistakes.yaml");
    assert_eq!(
        results.rules[0].target.as_deref(),
        Some("missing/tsconfig.json")
    );
    let direct = no_mistakes::codebase::rules::filesystem_dispatch::run_filesystem_rules(
        &fixture_root("auto-config-path"),
        None,
    )
    .unwrap();
    assert_eq!(direct, results.rules);

    let suppressed = run_all(fixture_root("auto-config-suppression"), None, None).unwrap();
    assert!(suppressed.rules.is_empty(), "{:?}", suppressed.rules);
    let direct_suppressed =
        no_mistakes::codebase::rules::filesystem_dispatch::run_filesystem_rules(
            &fixture_root("auto-config-suppression"),
            None,
        )
        .unwrap();
    assert!(direct_suppressed.is_empty(), "{direct_suppressed:?}");
}
