use super::*;

/// Write a minimal `.no-mistakes.yml` that enables all the given rule IDs.
/// `production-dependency-declarations` has no default `workspaceRoots` (it
/// is a required option), so it gets an explicit value here to stay in scope
/// for these "every rule dispatches cleanly" tests.
fn write_config(dir: &std::path::Path, rules: &[&str]) -> std::path::PathBuf {
    let rule_entries: String = rules
        .iter()
        .map(|id| {
            if *id == PRODUCTION_DEPENDENCY_DECLARATIONS {
                format!(
                    "  - rule: {id}\n    scope: repository\n    options:\n      \
                     workspaceRoots: [\".\"]\n"
                )
            } else {
                format!("  - rule: {id}\n    scope: repository\n")
            }
        })
        .collect();
    let yaml = format!("rules:\n{rule_entries}");
    let config_path = dir.join(".no-mistakes.yml");
    std::fs::write(&config_path, yaml).unwrap();
    config_path
}

/// Cover all dispatch branches via `run_filesystem_rules_with_files`.
/// Passing an empty file list means no rule actually does I/O on files —
/// they just enter their dispatch branch and return Ok(empty).
#[test]
fn dispatch_with_files_covers_all_rule_branches() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = write_config(tmp.path(), FILESYSTEM_RULE_IDS);
    let findings = run_filesystem_rules_with_files(tmp.path(), Some(&config_path), &[]).unwrap();
    // Empty file list → no findings; but all dispatch branches have been entered.
    assert!(
        findings.is_empty(),
        "empty file list should produce no findings: {findings:?}"
    );
}

#[test]
fn dispatch_with_files_returns_configuration_errors() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch/invalid-config");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let error = run_filesystem_rules_with_visible_and_snapshot(
        &root,
        Some(&root.join(".no-mistakes.yml")),
        &[],
        &snapshot,
    )
    .unwrap_err();
    assert!(error.to_string().contains("parse"), "{error:#}");
}

/// Cover all dispatch branches via `run_filesystem_rules`.
/// Each rule's own `check()` fn is called; with an empty/non-git directory
/// discover_files returns nothing, so no findings are emitted.
#[test]
fn dispatch_standalone_covers_all_rule_branches() {
    let tmp = tempfile::tempdir().unwrap();
    let config_path = write_config(tmp.path(), FILESYSTEM_RULE_IDS);
    let findings = run_filesystem_rules(tmp.path(), Some(&config_path)).unwrap();
    assert!(
        findings.is_empty(),
        "empty directory should produce no findings: {findings:?}"
    );
}

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
    let entrypoints = include_str!("entrypoints.rs");
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

#[test]
fn standalone_filesystem_rules_share_one_discovered_file_list() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-runner/facts-and-filesystem/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = root.join(".no-mistakes.yml");

    let loaded = crate::config::v2::load_v2_config(&root, Some(&config)).unwrap();
    let preserved_roots =
        preserved::filesystem_rule_target_roots(&root, &loaded, FILESYSTEM_RULE_IDS);
    let files = crate::codebase::ts_source::discover_files_preserving_roots(
        &root,
        &loaded.filesystem.skip_directories,
        &preserved_roots,
    );
    let expected = run_filesystem_rules_with_files(&root, Some(&config), &files).unwrap();
    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();

    let rules: Vec<&str> = findings
        .iter()
        .map(|finding| finding.rule.as_str())
        .collect();
    assert_eq!(
        rules,
        vec![RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_TESTS],
        "expected both enabled filesystem rules to run with deterministic output: {findings:#?}"
    );
    assert_eq!(
        findings, expected,
        "standalone dispatch should match one shared pre-discovered file list"
    );
}

#[test]
fn standalone_filesystem_rules_preserve_project_roots_under_skipped_dirs() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/filesystem-dispatch/project-under-skipped-dir/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = root.join(".no-mistakes.yml");

    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].rule, RUST_NO_INLINE_TESTS);
    assert_eq!(findings[0].file, "fixtures/app/src/lib.rs");
}

#[test]
fn standalone_banned_paths_adds_only_tracked_entries_from_source_skips() {
    let fixture = crate::test_support::materialize_gitignore_fixture("banned-paths-source-skips");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let output = std::process::Command::new("git")
        .current_dir(fixture.path())
        .args([
            "rm",
            "--cached",
            "--",
            "build/blocked.patch",
            "nested/blocked.patch",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let findings = run_filesystem_rules(fixture.path(), None).unwrap();
    let files = findings
        .iter()
        .filter(|finding| finding.rule == BANNED_PATHS)
        .map(|finding| finding.file.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        files,
        vec![
            "dist/blocked.patch",
            "fixtures/blocked.patch",
            "target/blocked.patch",
        ]
    );
}

#[test]
fn standalone_filesystem_rules_preserve_option_roots_without_leaking() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/filesystem-dispatch/option-root-under-skipped-dir/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = root.join(".no-mistakes.yml");

    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();

    let pairs: Vec<(&str, &str)> = findings
        .iter()
        .map(|finding| (finding.rule.as_str(), finding.file.as_str()))
        .collect();
    assert_eq!(
        pairs,
        vec![(RUST_NO_INLINE_TESTS, "fixtures/app/src/lib.rs")],
        "{findings:#?}"
    );
}

#[test]
fn forbidden_workspace_closure_preserves_repo_workspace_for_project_rules() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/filesystem-dispatch/forbidden-workspace-project-root/fixture",
    );
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = root.join(".no-mistakes.yml");

    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].rule, FORBIDDEN_WORKSPACE_CLOSURE);
    assert_eq!(findings[0].file, "packages/domain/package.json");
    assert_eq!(
        findings[0].import.as_deref(),
        Some("@acme/app -> @acme/domain -> @acme/secret")
    );
}

#[test]
fn combined_rust_rules_emit_all_configured_findings() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/filesystem-dispatch/rust-combined/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = root.join(".no-mistakes.yml");

    let findings = run_filesystem_rules(&root, Some(&config)).unwrap();
    let rules: Vec<&str> = findings
        .iter()
        .map(|finding| finding.rule.as_str())
        .collect();

    assert_eq!(
        rules,
        vec![
            RUST_MAX_LINES_PER_FILE,
            RUST_NO_INLINE_ALLOWS,
            RUST_NO_INLINE_TESTS,
        ],
        "{findings:#?}"
    );
    assert!(findings.iter().all(|finding| finding.file == "src/lib.rs"));
}

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
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/call-literals/valid"),
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

/// Cover the false branches of the `if rule_enabled(...)` guards for
/// `RUST_MAX_LINES_PER_FILE` and `RUST_NO_INLINE_TESTS` by running with a
/// config that omits those two rules, exercising the skip paths.
#[test]
fn dispatch_with_files_skips_disabled_rules() {
    let tmp = tempfile::tempdir().unwrap();
    // Omit RUST_MAX_LINES_PER_FILE and RUST_NO_INLINE_TESTS from the config.
    let rules_without_rust: Vec<&str> = FILESYSTEM_RULE_IDS
        .iter()
        .copied()
        .filter(|&r| r != RUST_MAX_LINES_PER_FILE && r != RUST_NO_INLINE_TESTS)
        .collect();
    let config_path = write_config(tmp.path(), &rules_without_rust);
    let findings = run_filesystem_rules_with_files(tmp.path(), Some(&config_path), &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn dispatch_with_files_keeps_supplied_banned_paths_authoritative() {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore/banned-paths-tracked-only");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let supplied = fixture.path().join("untracked-visible.patch");

    let findings = run_filesystem_rules_with_files(
        fixture.path(),
        Some(&fixture.path().join(".no-mistakes.yml")),
        std::slice::from_ref(&supplied),
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].rule, BANNED_PATHS);
    assert_eq!(findings[0].file, "untracked-visible.patch");
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

mod coverage;
