use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/structured-config-policy/ancestor-override-subset"),
    )
}

fn config() -> NoMistakesConfig {
    config_from(
        r#"
policies:
  - files: ["**/.config.json"]
    valueAssertions:
      - kind: ancestor-override-subset
"#,
    )
}

fn config_from(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn findings_for(paths: &[&str]) -> Vec<RuleFinding> {
    let root = fixture_root();
    let files = paths.iter().map(|path| root.join(path)).collect::<Vec<_>>();
    check_with_files(&root, &config(), &files).unwrap()
}

#[test]
fn reports_lost_direct_and_deep_ancestor_overrides() {
    let findings = findings_for(&[
        ".config.json",
        "direct/.config.json",
        "direct/only.ts",
        "direct/only.tsx",
        "deep/base/.config.json",
        "deep/middle/.config.json",
        "deep/middle/only.ts",
        "deep/target/.config.json",
    ]);
    let body = format!("{findings:?}");
    assert!(body.contains("direct/.config.json"), "{body}");
    assert!(body.contains("deep/middle/.config.json"), "{body}");
}

#[test]
fn accepts_restatements_and_matching_overrides() {
    let findings = findings_for(&[
        ".config.json",
        "restated/.config.json",
        "matching/.config.json",
    ]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn reports_missing_and_value_different_rules() {
    let findings = findings_for(&[
        ".config.json",
        "missing/.config.json",
        "missing/only.ts",
        "different/.config.json",
        "different/only.ts",
    ]);
    let body = format!("{findings:?}");
    assert!(body.contains("missing/.config.json"), "{body}");
    assert!(body.contains("different/.config.json"), "{body}");
}

#[test]
fn uses_actual_candidates_and_effective_glob_unions() {
    let root = fixture_root();
    let files = [
        root.join("union/base.json"),
        root.join("union/nested/.config.json"),
        root.join("union/nested/specific-file.ts"),
        root.join("union/nested/component.tsx"),
    ];
    let findings = check_with_files(&root, &config(), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn detects_lost_rules_for_actual_tsx_specific_and_nested_source_candidates() {
    let root = fixture_root();
    let files = [
        root.join("actual/base.json"),
        root.join("actual/nested/.config.json"),
        root.join("actual/nested/component.tsx"),
        root.join("actual/nested/specific-file.ts"),
        root.join("actual/nested/src/app.ts"),
        root.join("actual/sibling.ts"),
    ];
    let findings = check_with_files(&root, &config(), &files).unwrap();
    let body = format!("{findings:?}");
    assert!(body.contains("tsx-rule"), "{body}");
    assert!(body.contains("specific-rule"), "{body}");
    assert!(body.contains("source-rule"), "{body}");
    assert!(!body.contains("sibling-rule"), "{body}");
}

#[test]
fn respects_excluded_candidates_and_later_override_precedence() {
    let root = fixture_root();
    let files = [
        root.join("effective/base.json"),
        root.join("effective/nested/.config.json"),
        root.join("effective/nested/generated/skip.ts"),
        root.join("effective/nested/keep.ts"),
    ];
    let findings = check_with_files(&root, &config(), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn merges_lost_rules_in_ancestor_order() {
    let findings = findings_for(&[
        ".config.json",
        "merge/first/.config.json",
        "merge/second/.config.json",
        "merge/target/.config.json",
    ]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn reports_cycles_malformed_and_outside_extends_but_skips_packages() {
    let findings = findings_for(&[
        "cycle/a/.config.json",
        "cycle/b/.config.json",
        "malformed/.config.json",
        "outside/.config.json",
        "package/.config.json",
    ]);
    let body = format!("{findings:?}");
    assert!(body.contains("cycle/a/.config.json"), "{body}");
    assert!(body.contains("malformed/.config.json"), "{body}");
    assert!(body.contains("outside/.config.json"), "{body}");
    assert!(!body.contains("package/.config.json"), "{body}");
}

#[test]
fn supports_custom_structured_config_keys() {
    let root = fixture_root();
    let parent = root.join("custom/parent.json");
    let target = root.join("custom/target/config.json");
    let sources = super::super::source_store_for_files(&[parent.clone(), target.clone()]);
    let source = super::super::read_source(&sources, &target).unwrap();
    let value =
        crate::codebase::structured_value::parse_structured_value(&target, &source).unwrap();
    let assertion = ValueAssertion {
        kind: Some(AssertionKind::AncestorOverrideSubset),
        extends_key: "parents".to_string(),
        overrides_key: "scoped".to_string(),
        override_files_key: "paths".to_string(),
        override_rules_key: "policies".to_string(),
        ..Default::default()
    };
    let mut resolver = super::ancestor_override_subset::AncestorResolver::new(&root, &sources);
    let findings = super::ancestor_override_subset::check_ancestor_override_subset(
        "custom/target/config.json",
        &target,
        &[parent.clone(), target.clone()],
        &value,
        &assertion,
        &mut resolver,
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("custom-rule"), "{findings:?}");
}

#[test]
fn reports_invalid_extends_and_override_shapes_without_silently_skipping_them() {
    let root = fixture_root();
    let files = [
        root.join("malformed/.config.json"),
        root.join("invalid-override/base.json"),
        root.join("invalid-override/.config.json"),
        root.join("invalid-override/entry.ts"),
    ];
    let findings = check_with_files(&root, &config(), &files).unwrap();
    let body = format!("{findings:?}");
    assert!(body.contains("malformed/.config.json"), "{body}");
    assert!(body.contains("invalid-override/.config.json"), "{body}");
}
