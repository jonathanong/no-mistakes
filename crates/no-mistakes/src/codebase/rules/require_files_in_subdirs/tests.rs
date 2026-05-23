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
