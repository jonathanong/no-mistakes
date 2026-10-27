// Output cases moved to cli_rra_snapshot_structured_config_policy.
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
fn equals_file_rejects_paths_outside_the_repository_root() {
    let root = fixture_root("nested-plugins");
    let files = vec![root.join("pkg/.oxlintrc.json")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["pkg/.oxlintrc.json"]
    valueAssertions:
      - key: plugins
        kind: equals-file
        file: ../equals-file-selector/root.yml
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("outside the repository root"),
        "{findings:?}"
    );

    let symlink = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["pkg/.oxlintrc.json"]
    valueAssertions:
      - key: plugins
        kind: equals-file
        file: escape-link/root.yml
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(symlink.len(), 1, "{symlink:?}");
    assert!(
        symlink[0].message.contains("outside the repository root"),
        "{symlink:?}"
    );
}
