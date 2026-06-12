use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/config-path-references")
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
fn reports_missing_config_path_references() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("config/app.yml"),
        root.join("config/existing.json"),
        root.join("src/a.test.ts"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
files: [config/app.yml]
keys: [paths.required, paths.glob]
allowGlobs: true
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("missing.json"));
}

#[test]
fn glob_references_default_to_config_file_directory() {
    let root = fixture_root("fixture");
    let opts = Options {
        allow_globs: true,
        ..Default::default()
    };
    let rel_files = vec!["config/existing.json".to_string()];

    assert!(reference_exists(
        &root,
        &root.join("config/app.yml"),
        &opts,
        "*.json",
        &rel_files
    ));
}

#[test]
fn ignores_unreadable_or_invalid_config_files() {
    let root = fixture_root("fixture");
    let files = vec![root.join("config"), root.join("config/invalid.yml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
files: [config, config/invalid.yml]
keys: [paths.required]
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn extracts_only_string_or_sequence_values_at_config_keys() {
    let value: serde_yaml::Value = serde_yaml::from_str(
        r#"
paths:
  sequence: [one.json, 2]
  object:
    nested: true
"#,
    )
    .unwrap();

    assert_eq!(values_at_key(&value, "paths.missing"), Vec::<String>::new());
    assert_eq!(
        values_at_key(&value, "paths.sequence"),
        vec!["one.json".to_string()]
    );
    assert_eq!(values_at_key(&value, "paths.object"), Vec::<String>::new());
}

#[test]
fn can_resolve_references_from_repository_root() {
    let root = fixture_root("fixture");
    let opts = Options {
        base_dir: BaseDir::Root,
        ..Default::default()
    };

    assert!(reference_exists(
        &root,
        &root.join("config/app.yml"),
        &opts,
        "config/existing.json",
        &[]
    ));
}
