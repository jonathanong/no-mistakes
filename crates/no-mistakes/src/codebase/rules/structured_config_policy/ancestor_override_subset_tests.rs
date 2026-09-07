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
        "deep/base/.config.json",
        "deep/middle/.config.json",
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
        "different/.config.json",
    ]);
    let body = format!("{findings:?}");
    assert!(body.contains("missing/.config.json"), "{body}");
    assert!(body.contains("different/.config.json"), "{body}");
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
    let sources = super::super::source_store_for_files(&[parent, target.clone()]);
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
    let findings = super::ancestor_override_subset::check_ancestor_override_subset(
        &root,
        "custom/target/config.json",
        &target,
        &sources,
        &value,
        &assertion,
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("custom-rule"), "{findings:?}");
}
