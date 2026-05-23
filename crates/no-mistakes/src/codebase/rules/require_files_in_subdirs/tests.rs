use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn config_with_rule(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fixture(path: &str) -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/require-files-in-subdirs")
        .join(path)
}

#[test]
fn pass_fixture_no_findings() {
    let root = fixture("pass");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files: Vec<PathBuf> = [
        root.join("queues/email/package.json"),
        root.join("queues/email/queues.mts"),
        root.join("queues/email/enqueues.mts"),
    ]
    .into();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "expected no findings, got: {findings:?}"
    );
}

#[test]
fn fail_fixture_reports_missing_files() {
    let root = fixture("fail");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files: Vec<PathBuf> = [root.join("queues/email/package.json")].into();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(
        findings.len(),
        2,
        "expected two findings, got: {findings:?}"
    );
    let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
    assert!(
        messages.iter().any(|m| m.contains("queues.mts")),
        "expected queues.mts missing, got: {messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("enqueues.mts")),
        "expected enqueues.mts missing, got: {messages:?}"
    );
}

#[test]
fn no_op_when_packages_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_rule("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn reports_missing_required_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let pkg = root.join("queues/email/package.json");
    std::fs::create_dir_all(pkg.parent().unwrap()).unwrap();
    std::fs::write(&pkg, "{}").unwrap();
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json, queues.mts]";
    let config = config_with_rule(yaml);
    let files = vec![pkg];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("queues.mts"));
    assert_eq!(findings[0].line, 1);
}

#[test]
fn no_finding_when_all_required_present() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let pkg = root.join("queues/email/package.json");
    let qts = root.join("queues/email/queues.mts");
    std::fs::create_dir_all(pkg.parent().unwrap()).unwrap();
    std::fs::write(&pkg, "{}").unwrap();
    std::fs::write(&qts, "").unwrap();
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json, queues.mts]";
    let config = config_with_rule(yaml);
    let files = vec![pkg, qts];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn require_any_of_passes_when_one_present() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let enq = root.join("queues/email/enqueues.mts");
    std::fs::create_dir_all(enq.parent().unwrap()).unwrap();
    std::fs::write(&enq, "").unwrap();
    let yaml =
        "packages:\n  - root: queues\n    requireAnyOf:\n      - [enqueues.mts, consumers.mts]";
    let config = config_with_rule(yaml);
    let files = vec![enq];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn require_any_of_fails_when_none_present() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let pkg = root.join("queues/email/package.json");
    std::fs::create_dir_all(pkg.parent().unwrap()).unwrap();
    std::fs::write(&pkg, "{}").unwrap();
    let yaml =
        "packages:\n  - root: queues\n    requireAnyOf:\n      - [enqueues.mts, consumers.mts]";
    let config = config_with_rule(yaml);
    let files = vec![pkg];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("enqueues.mts"));
}

#[test]
fn glob_pattern_in_require_any_of() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let f = root.join("queues/email/enqueues.mts");
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, "").unwrap();
    let yaml = "packages:\n  - root: queues\n    requireAnyOf:\n      - [\"*.mts\"]";
    let config = config_with_rule(yaml);
    let files = vec![f];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(findings.is_empty(), "glob *.mts should match enqueues.mts");
}

#[test]
fn root_level_files_not_treated_as_subdirs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let f = root.join("queues/direct-file.mts");
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, "").unwrap();
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    let files = vec![f];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "direct children of root have no first-level subdir, got: {findings:?}"
    );
}

#[test]
fn check_with_files_empty_packages_is_noop() {
    // Exercises the opts.packages.is_empty() branch (line 50) in check_with_files.
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_rule("{}");
    let findings = check_with_files(tmp.path(), &config, &[]).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_standalone_with_packages_discovers_files() {
    // Exercises lines 34-35 of check(): non-empty packages → discover_files.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let pkg = root.join("queues/email/package.json");
    std::fs::create_dir_all(pkg.parent().unwrap()).unwrap();
    std::fs::write(&pkg, "{}").unwrap();
    // This directory is not a git repo so discover_files returns nothing,
    // meaning no subdirs are found and no findings are produced.
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    let findings = check(root, &config).unwrap();
    // discover_files returns empty outside a git repo, so no findings.
    assert!(findings.is_empty());
}

#[test]
fn absolute_package_root_is_used_directly() {
    // Exercises spec.root.is_absolute() branch (line 64).
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let abs_queues = root.join("queues");
    let pkg = abs_queues.join("email/package.json");
    std::fs::create_dir_all(pkg.parent().unwrap()).unwrap();
    std::fs::write(&pkg, "{}").unwrap();
    // Use absolute path for the package root
    let yaml = format!(
        "packages:\n  - root: {}\n    requiredFiles: [queues.mts]",
        abs_queues.display()
    );
    let config = config_with_rule(&yaml);
    let files = vec![pkg];
    let findings = check_with_files(root, &config, &files).unwrap();
    // Should report missing queues.mts
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("queues.mts"));
}

#[test]
fn glob_pattern_in_require_any_of_matched_returns_early() {
    // Exercises the `return Ok(true)` (line 129) when glob matches.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let f = root.join("queues/email/enqueues.mts");
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, "").unwrap();
    let yaml = "packages:\n  - root: queues\n    requireAnyOf:\n      - [\"*.mts\", other.mts]";
    let config = config_with_rule(yaml);
    let files = vec![f];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(findings.is_empty(), "glob match should return true early");
}

#[test]
fn files_outside_root_are_skipped_in_first_level_subdirs() {
    // Exercises the strip_prefix error branch (line 145): files not under root are
    // skipped by first_level_subdirs.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let other_root = tempfile::tempdir().unwrap();
    let outside_file = other_root.path().join("unrelated/something.ts");
    std::fs::create_dir_all(outside_file.parent().unwrap()).unwrap();
    std::fs::write(&outside_file, "").unwrap();
    let pkg = root.join("queues/email/package.json");
    std::fs::create_dir_all(pkg.parent().unwrap()).unwrap();
    std::fs::write(&pkg, "{}").unwrap();
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    // Include both the in-root file and an out-of-root file
    let files = vec![pkg, outside_file];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "email subdir has package.json, should pass; outside file is skipped"
    );
}

#[test]
fn spec_root_itself_in_file_list_is_skipped_by_first_level_subdirs() {
    // Exercises line 149: when rel is empty (file == spec_root), components.next()
    // returns None and the iteration skips without recording a subdir.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let queues = root.join("queues");
    std::fs::create_dir_all(&queues).unwrap();
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    // Pass the spec root directory itself in the file list — strip_prefix returns ""
    let files = vec![queues];
    let findings = check_with_files(root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "spec root dir in file list should produce no subdirs and no findings"
    );
}

#[test]
fn glob_pattern_not_matched_falls_through_to_next_group_entry() {
    // Exercises line 130 (closing `}` of `if matched { return Ok(true); }`):
    // the glob is checked but does NOT match any file, so `matched = false`
    // and the `if` block is skipped; execution falls through to the next pattern.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // Create a file that does NOT match the first glob *.ts, but DOES match *.mts
    let f = root.join("queues/email/something.mts");
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(&f, "").unwrap();
    // Two-pattern group: first *.ts won't match (no .ts files), second *.mts will.
    let yaml = "packages:\n  - root: queues\n    requireAnyOf:\n      - [\"*.ts\", \"*.mts\"]";
    let config = config_with_rule(yaml);
    let files = vec![f];
    let findings = check_with_files(root, &config, &files).unwrap();
    // The second pattern *.mts matches, so no finding.
    assert!(
        findings.is_empty(),
        "second glob *.mts should satisfy the group requirement"
    );
}
