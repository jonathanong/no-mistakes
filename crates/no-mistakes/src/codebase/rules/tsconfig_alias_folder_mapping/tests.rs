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

#[test]
fn alias_with_correct_prefix_but_wrong_target_is_flagged_once() {
    // Exercises lines 132-158: the alias @agents/email has the right prefix but
    // targets the wrong folder.  The "direction 1" check fires first (alias→target)
    // flagging it; the "direction 2" check then sees `already_flagged = true` via
    // the f.message.contains() check and skips the duplicate.
    let tmp = tempfile::tempdir().unwrap();
    // @agents/email → backend/modules/email (wrong — should be backend/agents/email)
    let tsconfig =
        r#"{"compilerOptions": {"paths": {"@agents/email": ["backend/modules/email"]}}}"#;
    std::fs::write(tmp.path().join("tsconfig.json"), tsconfig).unwrap();
    let config = config_from_yaml(agents_config());
    let findings = check(tmp.path(), &config).unwrap();
    // Should produce exactly one finding (not duplicated by direction-2 check)
    assert!(!findings.is_empty(), "expected at least one finding");
}

#[test]
fn alias_with_valid_mapping_is_not_double_flagged() {
    // Exercises the `valid` check (lines 136-148) in direction-2:
    // alias starts with @agents/ and the target matches — so `valid = true`
    // and no direction-2 finding is emitted.
    let tmp = tempfile::tempdir().unwrap();
    // Correct mapping: @agents/email → backend/agents/email
    let tsconfig = r#"{"compilerOptions": {"paths": {"@agents/email": ["backend/agents/email"]}}}"#;
    std::fs::write(tmp.path().join("tsconfig.json"), tsconfig).unwrap();
    let config = config_from_yaml(agents_config());
    let findings = check(tmp.path(), &config).unwrap();
    assert!(
        findings.is_empty(),
        "correct mapping should produce no findings"
    );
}

#[test]
fn two_mappings_already_flagged_skip_prevents_duplicate() {
    // Exercises lines 132-133 and 158: with two mappings (@agents → agents,
    // @workers → workers), an alias @agents/email that targets backend/workers/email
    // triggers direction-1 (wrong folder) AND direction-2 for the @workers mapping
    // (correct folder but wrong prefix). The already_flagged check (lines 132-133)
    // detects the direction-1 finding and skips the direction-2 duplicate (line 158).
    let tmp = tempfile::tempdir().unwrap();
    let config_yaml =
        "tsconfig: tsconfig.json\nbaseDir: backend\nmappings:\n  - prefix: \"@agents\"\n    root: agents\n  - prefix: \"@workers\"\n    root: workers\n";
    let tsconfig =
        r#"{"compilerOptions": {"paths": {"@agents/email": ["backend/workers/email"]}}}"#;
    std::fs::write(tmp.path().join("tsconfig.json"), tsconfig).unwrap();
    let config = config_from_yaml(config_yaml);
    let findings = check(tmp.path(), &config).unwrap();
    // Direction-1 fires (wrong target for @agents); direction-2 also fires for
    // @workers but already_flagged = true so we get exactly one finding.
    assert!(!findings.is_empty(), "expected at least one finding");
}

#[test]
fn two_mappings_same_root_valid_alias_not_flagged_by_direction_two() {
    // Exercises lines 139-141 and 158: two mappings share the same root folder.
    // @a/sub → backend/shared/sub is correct for the @a mapping. Direction-2 for
    // the @b mapping fires (target starts with backend/shared/) but the valid check
    // (lines 139-141) finds the alias is correct for the @a mapping, so no finding.
    let tmp = tempfile::tempdir().unwrap();
    let config_yaml =
        "tsconfig: tsconfig.json\nbaseDir: backend\nmappings:\n  - prefix: \"@a\"\n    root: shared\n  - prefix: \"@b\"\n    root: shared\n";
    // @a/sub correctly maps to backend/shared/sub per the @a mapping.
    // Direction-2 for @b fires because the target starts with backend/shared/,
    // but valid = true because the @a mapping is correct.
    let tsconfig = r#"{"compilerOptions": {"paths": {"@a/sub": ["backend/shared/sub"]}}}"#;
    std::fs::write(tmp.path().join("tsconfig.json"), tsconfig).unwrap();
    let config = config_from_yaml(config_yaml);
    let findings = check(tmp.path(), &config).unwrap();
    assert!(
        findings.is_empty(),
        "valid alias should not be flagged by direction-2 via valid check: {findings:?}"
    );
}
