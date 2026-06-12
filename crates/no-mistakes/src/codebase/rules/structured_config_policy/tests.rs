use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/structured-config-policy")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn reports_required_and_banned_structured_config_keys() {
    let root = fixture_root("fixture");
    let files = vec![root.join("app.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [app.yml]
    requiredKeys: [runtime.version, runtime.owner]
    bannedKeys: [legacy.enabled]
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("runtime.owner")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("legacy.enabled")));
}

#[test]
fn ignores_invalid_structured_config_files() {
    let root = fixture_root("fixture");
    let files = vec![root.join("invalid.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [invalid.yml]
    requiredKeys: [runtime.version]
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn ignores_unreadable_structured_config_paths() {
    let root = fixture_root("fixture");
    let files = vec![root.clone()];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["."]
    requiredKeys: [runtime.version]
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}
