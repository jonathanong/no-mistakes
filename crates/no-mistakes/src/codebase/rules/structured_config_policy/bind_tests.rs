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

fn bind_override_yaml() -> &'static str {
    r#"
policies:
  - files: ["*.json"]
    valueAssertions:
      - key: overrides.[].rules.no-restricted-properties.[]
        kind: object-shape
        match: any
        requiredKeys: [message]
        forbiddenKeys: [object]
        requiredValues:
          property: bind
"#
}

#[test]
fn not_single_file_strips_star_star_slash_prefix() {
    let root = fixture_root("prefix-single-file");
    let files = vec![root.join("app.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: [app.yml]
    valueAssertions:
      - key: overrides.[].files.[]
        kind: not-single-file
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("single-file entry"));
}

#[test]
fn match_any_fails_per_override_that_drops_bind() {
    let root = fixture_root("overrides-bind");
    let files = vec![root.join("drop.json")];
    let findings = check_with_files(&root, &config(bind_override_yaml()), &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("at least one"));
}

#[test]
fn match_any_skips_overrides_that_omit_the_rule() {
    let root = fixture_root("overrides-bind");
    let files = vec![root.join("omit.json")];
    let findings = check_with_files(&root, &config(bind_override_yaml()), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
