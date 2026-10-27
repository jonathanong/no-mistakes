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
    crate::codebase::ts_resolver::normalize_path(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/require-files-in-subdirs/fixture")
            .join(path),
    )
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
fn nested_direct_child_glob_fixture_passes() {
    let root = fixture("nested-direct-child-pass");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files: Vec<PathBuf> = [
        root.join("packages/email/package.json"),
        root.join("packages/email/enqueues/email.mts"),
        root.join("packages/email/workers/email.mts"),
    ]
    .into();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "nested direct-child globs should match package-relative files: {findings:?}"
    );
}

#[test]
fn nested_direct_child_glob_fixture_reports_missing_group() {
    let root = fixture("nested-direct-child-fail");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files: Vec<PathBuf> = [
        root.join("packages/email/package.json"),
        root.join("packages/email/enqueues/email.mts"),
        root.join("packages/email/workers/nested/email.mts"),
    ]
    .into();
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "expected one finding, got: {findings:?}");
    assert!(
        findings[0].message.contains("workers/*.mts"),
        "{}",
        findings[0].message
    );
}

#[test]
fn bracketed_required_file_is_treated_as_literal_path() {
    let root = fixture("bracketed-route");
    let route = root.join("routes/users/pages/[id].tsx");
    let yaml = "packages:\n  - root: routes\n    requiredFiles: [\"pages/[id].tsx\"]";
    let config = config_with_rule(yaml);
    let findings = check_with_files(&root, &config, &[route]).unwrap();
    assert!(
        findings.is_empty(),
        "bracketed route filename should be matched as an exact path: {findings:?}"
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
    let root = fixture("email-package-only");
    let pkg = root.join("queues/email/package.json");
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json, queues.mts]";
    let config = config_with_rule(yaml);
    let files = vec![pkg];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("queues.mts"));
    assert_eq!(findings[0].line, 1);
}

#[test]
fn no_finding_when_all_required_present() {
    let root = fixture("email-package-and-queues");
    let pkg = root.join("queues/email/package.json");
    let qts = root.join("queues/email/queues.mts");
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json, queues.mts]";
    let config = config_with_rule(yaml);
    let files = vec![pkg, qts];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn require_any_of_passes_when_one_present() {
    let root = fixture("email-enqueues");
    let enq = root.join("queues/email/enqueues.mts");
    let yaml =
        "packages:\n  - root: queues\n    requireAnyOf:\n      - [enqueues.mts, consumers.mts]";
    let config = config_with_rule(yaml);
    let files = vec![enq];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn require_any_of_fails_when_none_present() {
    let root = fixture("email-package-only");
    let pkg = root.join("queues/email/package.json");
    let yaml =
        "packages:\n  - root: queues\n    requireAnyOf:\n      - [enqueues.mts, consumers.mts]";
    let config = config_with_rule(yaml);
    let files = vec![pkg];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("enqueues.mts"));
}

#[test]
fn glob_pattern_in_require_any_of() {
    let root = fixture("email-enqueues");
    let f = root.join("queues/email/enqueues.mts");
    let yaml = "packages:\n  - root: queues\n    requireAnyOf:\n      - [\"*.mts\"]";
    let config = config_with_rule(yaml);
    let files = vec![f];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "glob *.mts should match enqueues.mts");
}

#[test]
fn root_level_files_not_treated_as_subdirs() {
    let root = fixture("direct-child");
    let f = root.join("queues/direct-file.mts");
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    let files = vec![f];
    let findings = check_with_files(&root, &config, &files).unwrap();
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
    // Copy the saved tree into a temp dir outside this checkout. `git ls-files`
    // fails there, so discovery takes the non-git fallback walk this test used
    // before the fixture lived inside the repo. package.json is present, so
    // that walk produces no finding.
    let tmp = crate::test_support::materialize_saved_fixture(&fixture("email-package-only"));
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn absolute_package_root_is_used_directly() {
    // Exercises spec.root.is_absolute() branch (line 64).
    let root = fixture("email-package-only");
    let abs_queues = root.join("queues");
    let pkg = abs_queues.join("email/package.json");
    // Use absolute path for the package root
    let yaml = format!(
        "packages:\n  - root: {}\n    requiredFiles: [queues.mts]",
        abs_queues.display()
    );
    let config = config_with_rule(&yaml);
    let files = vec![pkg];
    let findings = check_with_files(&root, &config, &files).unwrap();
    // Should report missing queues.mts
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("queues.mts"));
}

#[test]
fn glob_pattern_in_require_any_of_matched_returns_early() {
    // Exercises the `return Ok(true)` (line 129) when glob matches.
    let root = fixture("email-enqueues");
    let f = root.join("queues/email/enqueues.mts");
    let yaml = "packages:\n  - root: queues\n    requireAnyOf:\n      - [\"*.mts\", other.mts]";
    let config = config_with_rule(yaml);
    let files = vec![f];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "glob match should return true early");
}

#[test]
fn files_outside_root_are_skipped_in_first_level_subdirs() {
    // Exercises the strip_prefix error branch (line 145): files not under root are
    // skipped by first_level_subdirs.
    let bundle = fixture("outside-and-in");
    let root = bundle.join("root");
    let outside_file = bundle.join("outside/unrelated/something.ts");
    let pkg = root.join("queues/email/package.json");
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    // Include both the in-root file and an out-of-root file
    let files = vec![pkg, outside_file];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "email subdir has package.json, should pass; outside file is skipped"
    );
}

#[test]
fn spec_root_itself_in_file_list_is_skipped_by_first_level_subdirs() {
    // Exercises line 149: when rel is empty (file == spec_root), components.next()
    // returns None and the iteration skips without recording a subdir.
    let root = fixture("spec-root-itself");
    let queues = root.join("queues");
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    // Pass the spec root directory itself in the file list — strip_prefix returns ""
    let files = vec![queues];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "spec root dir in file list should produce no subdirs and no findings"
    );
}

#[test]
fn files_in_root_but_outside_spec_subroot_skip_strip_prefix() {
    // Exercises line 151: strip_prefix(abs_root) fails for files that are inside the
    // top-level root (passing target_roots) but outside the spec's sub-root.
    let root = fixture("outside-spec");
    // This file is under root but NOT under root/queues — strip_prefix(root/queues) fails.
    let outside_spec = root.join("other-dir/file.ts");
    let pkg = root.join("queues/email/package.json");
    let yaml = "packages:\n  - root: queues\n    requiredFiles: [package.json]";
    let config = config_with_rule(yaml);
    let files = vec![outside_spec, pkg];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings.is_empty(),
        "email subdir has package.json; outside-spec file is skipped"
    );
}

#[test]
fn glob_pattern_not_matched_falls_through_to_next_group_entry() {
    // Exercises line 130 (closing `}` of `if matched { return Ok(true); }`):
    // the glob is checked but does NOT match any file, so `matched = false`
    // and the `if` block is skipped; execution falls through to the next pattern.
    let root = fixture("email-something-mts");
    // A file that does NOT match the first glob *.ts, but DOES match *.mts
    let f = root.join("queues/email/something.mts");
    // Two-pattern group: first *.ts won't match (no .ts files), second *.mts will.
    let yaml = "packages:\n  - root: queues\n    requireAnyOf:\n      - [\"*.ts\", \"*.mts\"]";
    let config = config_with_rule(yaml);
    let files = vec![f];
    let findings = check_with_files(&root, &config, &files).unwrap();
    // The second pattern *.mts matches, so no finding.
    assert!(
        findings.is_empty(),
        "second glob *.mts should satisfy the group requirement"
    );
}

#[test]
fn parent_segment_root_reports_missing_file() {
    // `services/../agents` must resolve to `agents` before strip_prefix.
    let root = fixture("parent-segment");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = vec![root.join("agents/email/index.mts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("agents/email"), "{findings:?}");
    assert!(findings[0].message.contains("README.md"), "{findings:?}");
    assert!(!findings[0].message.contains("escapes"));
}

#[test]
fn dot_prefix_root_still_matches() {
    let root = fixture("parent-segment");
    let config = config_with_rule("packages:\n  - root: ./agents\n    requiredFiles: [README.md]");
    let file = root.join("agents/email/index.mts");
    let findings = check_with_files(&root, &config, &[file]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("agents/email"), "{findings:?}");
    assert!(!findings[0].message.contains("escapes"));
}

#[test]
fn escaping_root_is_one_finding_and_skips_outside_files() {
    let root = fixture("escape");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let inside = root.join("agents/email/index.mts");
    let outside = fixture("fail").join("queues/email/package.json");
    let opts: Options = config.rule_applications(RULE_ID)[0]
        .try_rule_options()
        .unwrap();
    let findings = super::scan(&root, &config, &opts, &[inside.clone(), outside.clone()]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("escapes the repository"),
        "{findings:?}"
    );
    let outside_text = outside.display().to_string();
    assert!(
        findings.iter().all(|finding| {
            !finding.message.contains("missing required file")
                && !finding.file.contains("index.mts")
                && !finding.message.contains(&outside_text)
                && !finding.file.contains(&outside_text)
        }),
        "{findings:?}"
    );
    let checked = check_with_files(&root, &config, &[inside]).unwrap();
    assert_eq!(checked.len(), 1, "{checked:?}");
    assert!(checked[0].message.contains("escapes the repository"));
}
