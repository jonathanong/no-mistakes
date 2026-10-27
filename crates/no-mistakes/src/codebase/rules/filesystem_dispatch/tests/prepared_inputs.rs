use super::*;

#[test]
fn prebuilt_snapshot_catalog_entrypoint_accepts_an_empty_catalog() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-runner/empty");
    let config = crate::config::v2::NoMistakesConfig::default();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &[]);

    let findings = run_filesystem_rules_with_config_snapshot_and_vitest_catalog(
        &root,
        &config,
        &[],
        &snapshot,
        None,
    )
    .unwrap();

    assert!(findings.is_empty());
}

#[test]
fn prepared_dispatch_rejects_tsconfig_gate_without_workflow_documents() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/tsconfig-gate-coverage/missing-ci");
    let config_path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &[]);

    let error = super::run_filesystem_rules_with_config_snapshot_catalog_and_sources(
        &root,
        &config,
        &[],
        PreparedFilesystemRuleInputs {
            snapshot: &snapshot,
            next_config_resolver: None,
            vitest_catalog: None,
            sources: snapshot.source_store_for(&root),
            workflow_documents: None,
            tsconfig_gate_project_inputs: None,
            config_path: Some(&config_path),
        },
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("prepared workflow documents and project inputs are required"),
        "{error:#}"
    );
}

#[test]
fn pre_discovered_entrypoints_do_not_start_another_discovery_snapshot() {
    let entrypoints = include_str!("../entrypoints.rs");
    assert_eq!(
        entrypoints.matches("VisiblePathSnapshot::new(").count(),
        1,
        "only the standalone entrypoint may discover paths"
    );
}

#[test]
fn visible_snapshot_entrypoint_excludes_untracked_markdown() {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch/markdown-visible-tracked");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let output = std::process::Command::new("git")
        .current_dir(fixture.path())
        .args(["rm", "--cached", "--", "untracked.md"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(fixture.path());
    let findings = crate::codebase::rules::run_filesystem_rules_with_visible_and_snapshot(
        fixture.path(),
        Some(&fixture.path().join(".no-mistakes.yml")),
        &[
            fixture.path().join("CLAUDE.md"),
            fixture.path().join("tracked.md"),
            fixture.path().join("untracked.md"),
        ],
        &snapshot,
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn dispatcher_uses_scoped_baseline_inventory_for_markdown_rules() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/filesystem-dispatch/markdown-rules/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let files = [
        "docs/CLAUDE.md",
        "docs/tracked.md",
        "docs/over-budget.md",
        "baselines/reachability.json",
        "baselines/structure.json",
    ]
    .map(|file| root.join(file));
    let findings =
        run_filesystem_rules_with_files(&root, Some(&root.join(".no-mistakes.yml")), &files)
            .unwrap();

    assert!(
        findings.is_empty(),
        "tracked baselines outside the scoped docs project must remain available: {findings:#?}"
    );
}

#[test]
fn enabling_mermaid_validation_preserves_existing_markdown_findings() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/filesystem-dispatch/markdown-rules/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config_path = root.join(".no-mistakes.yml");
    let baseline = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let mut additive = baseline.clone();
    additive.rules.push(crate::config::v2::schema::RuleDef {
        rule: MARKDOWN_MERMAID_VALIDATION.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        ..Default::default()
    });
    let files = [
        "docs/CLAUDE.md",
        "docs/tracked.md",
        "docs/over-budget.md",
        "baselines/reachability.json",
        "baselines/structure.json",
    ]
    .map(|file| root.join(file));
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let run = |config| {
        run_filesystem_rules_with_config_snapshot_catalog_and_sources(
            &root,
            config,
            &files,
            PreparedFilesystemRuleInputs {
                snapshot: &snapshot,
                next_config_resolver: None,
                vitest_catalog: None,
                sources: snapshot.source_store_for(&root),
                workflow_documents: None,
                tsconfig_gate_project_inputs: None,
                config_path: Some(&config_path),
            },
        )
        .unwrap()
    };

    let expected = run(&baseline);
    let actual = run(&additive)
        .into_iter()
        .filter(|finding| finding.rule != MARKDOWN_MERMAID_VALIDATION)
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}
