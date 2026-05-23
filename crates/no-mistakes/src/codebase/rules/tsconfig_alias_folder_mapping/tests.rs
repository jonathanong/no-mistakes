use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/tsconfig-alias-folder-mapping")
        .join(path)
}

fn config_from_yaml(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn agents_config() -> &'static str {
    "tsconfig: tsconfig.json\nbaseDir: backend\nmappings:\n  - prefix: \"@agents\"\n    root: agents\n"
}

#[test]
fn pass_fixture_produces_no_findings() {
    let root = fixture("pass");
    let config = config_from_yaml(agents_config());
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty(), "expected no findings: {findings:?}");
}

#[test]
fn fail_fixture_produces_findings() {
    let root = fixture("fail");
    let config = config_from_yaml(agents_config());
    let findings = check(&root, &config).unwrap();
    assert!(!findings.is_empty(), "expected at least one finding");
    assert!(
        findings[0].message.contains("must target"),
        "unexpected message: {}",
        findings[0].message
    );
}

#[test]
fn no_options_is_noop() {
    let root = fixture("fail");
    let config = config_from_yaml("{}");
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn missing_tsconfig_is_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_from_yaml(
        "tsconfig: nonexistent.json\nbaseDir: backend\nmappings:\n  - prefix: \"@a\"\n    root: a\n",
    );
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn check_with_files_delegates_to_tsconfig() {
    let root = fixture("fail");
    let config = config_from_yaml(agents_config());
    // Pass an empty file list — rule reads tsconfig directly
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert!(!findings.is_empty());
}

#[test]
fn finding_has_correct_line_and_file() {
    let root = fixture("fail");
    let config = config_from_yaml(agents_config());
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings[0].line, 1);
    assert!(findings[0].file.contains("tsconfig.json"));
}

#[test]
fn multiple_targets_all_checked() {
    let tmp = tempfile::tempdir().unwrap();
    // Two targets, first wrong, second correct
    let tsconfig = r#"{"compilerOptions": {"paths": {"@agents/email": ["backend/modules/email", "backend/agents/email"]}}}"#;
    std::fs::write(tmp.path().join("tsconfig.json"), tsconfig).unwrap();
    let config = config_from_yaml(agents_config());
    let findings = check(tmp.path(), &config).unwrap();
    // The wrong target should be flagged
    assert!(
        findings
            .iter()
            .any(|f| f.message.contains("backend/modules/email")),
        "expected finding for wrong target"
    );
}

#[test]
fn no_findings_when_no_compiler_options_paths() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("tsconfig.json"),
        r#"{"compilerOptions": {}}"#,
    )
    .unwrap();
    let config = config_from_yaml(agents_config());
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn wrong_prefix_for_target_is_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    // The alias uses @wrong prefix but the target is in backend/agents/
    let tsconfig = r#"{"compilerOptions": {"paths": {"@wrong/email": ["backend/agents/email"]}}}"#;
    std::fs::write(tmp.path().join("tsconfig.json"), tsconfig).unwrap();
    let config = config_from_yaml(agents_config());
    let findings = check(tmp.path(), &config).unwrap();
    assert!(
        !findings.is_empty(),
        "expected finding for wrong prefix alias"
    );
}
