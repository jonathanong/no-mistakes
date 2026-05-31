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

    let files = crate::codebase::ts_source::discover_files(&root, &[]);
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
