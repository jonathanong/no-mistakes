use super::*;

/// Write a minimal `.no-mistakes.yml` that enables all the given rule IDs.
fn write_config(dir: &std::path::Path, rules: &[&str]) -> std::path::PathBuf {
    let rule_entries: String = rules
        .iter()
        .map(|id| format!("  - rule: {id}\n    scope: repository\n"))
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
fn aggregate_drops_exclusive_rust_sources_without_global_suppression_rereads() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/filesystem-dispatch/rust-combined/fixture");
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
        &snapshot,
        None,
        std::sync::Arc::clone(&sources),
    )
    .unwrap();

    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert_eq!(sources.physical_read_count(), 0);
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
        &snapshot,
        None,
        std::sync::Arc::clone(&sources),
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "placeholder.ts");
    assert_eq!(sources.physical_read_count(), 1);
}
