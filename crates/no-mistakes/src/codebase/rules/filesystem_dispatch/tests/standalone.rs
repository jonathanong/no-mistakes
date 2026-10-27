use super::*;

#[test]
fn standalone_filesystem_rules_share_one_discovered_file_list() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-runner/facts-and-filesystem/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = root.join(".no-mistakes.yml");

    let loaded = crate::config::v2::load_v2_config(&root, Some(&config)).unwrap();
    let preserved_roots =
        preserved::filesystem_rule_target_roots(&root, &loaded, FILESYSTEM_RULE_IDS).unwrap();
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
