use super::*;

#[test]
fn aggregate_reads_rust_sources_once_without_global_suppression_rereads() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/filesystem-dispatch/rust-combined/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    // Aggregate fact collection warms this request-owned store before the
    // filesystem dispatcher. The Rust rules must reuse that source and avoid
    // a second read for final suppression accounting.
    sources.read_path(&root.join("src/lib.rs")).unwrap();
    assert_eq!(sources.physical_read_count(), 1);

    let findings = run_filesystem_rules_with_config_snapshot_catalog_and_sources(
        &root,
        &config,
        &files,
        PreparedFilesystemRuleInputs {
            snapshot: &snapshot,
            next_config_resolver: None,
            vitest_catalog: None,
            sources: std::sync::Arc::clone(&sources),
            workflow_documents: None,
            tsconfig_gate_project_inputs: None,
            config_path: None,
        },
    )
    .unwrap();

    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn legacy_prepared_dispatcher_prepares_finite_set_call_facts_once() {
    // Keep this root private: process-wide counting must include worker parses without
    // admitting unrelated finite-set tests that run concurrently on shared fixtures.
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/prepared-dispatcher-count"),
    );
    let files = vec![root.join("schedules.mts"), root.join("registry.mts")];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: FINITE_SET_CONSISTENCY.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        options: serde_yaml::from_str(
            r#"
sets:
  - name: schedulerIds
    file: schedules.mts
    kind: ts-call-first-string-argument
    target: ai_agents.upsertJobScheduler
  - name: registryIds
    file: registry.mts
    kind: ts-const-array-property
    target: AI_AGENTS_SCHEDULED_JOBS
    property: id
comparisons:
  - left: schedulerIds
    right: registryIds
"#,
        )
        .unwrap(),
        ..Default::default()
    });

    crate::ast::begin_parse_count(&root);
    let findings = run_filesystem_rules_with_config_snapshot_catalog_and_sources(
        &root,
        &config,
        &files,
        PreparedFilesystemRuleInputs {
            snapshot: &snapshot,
            next_config_resolver: None,
            vitest_catalog: None,
            sources,
            workflow_documents: None,
            tsconfig_gate_project_inputs: None,
            config_path: None,
        },
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    assert_eq!(counts.get(&root.join("schedules.mts")), Some(&1));
    assert_eq!(counts.len(), 1, "{counts:?}");
}

#[test]
fn aggregate_finding_and_suppression_share_one_physical_read() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-empty-or-comments-only-files/fixture/fail");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);

    let findings = run_filesystem_rules_with_config_snapshot_catalog_and_sources(
        &root,
        &config,
        &files,
        PreparedFilesystemRuleInputs {
            snapshot: &snapshot,
            next_config_resolver: None,
            vitest_catalog: None,
            sources: std::sync::Arc::clone(&sources),
            workflow_documents: None,
            tsconfig_gate_project_inputs: None,
            config_path: None,
        },
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "placeholder.ts");
    assert_eq!(sources.physical_read_count(), 1);
}
