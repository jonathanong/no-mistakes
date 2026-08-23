use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/finite-set-consistency")
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
fn string_union_min_size_fails_closed_when_the_target_is_missing() {
    let root = fixture_root("fixture");
    let files = vec![root.join("src/types.ts")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sets:
  - name: missingUnion
    file: src/types.ts
    kind: ts-string-union
    target: MissingRouteName
    minSize: 1
comparisons:
  - left: missingUnion
    right: missingUnion
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0]
            .message
            .contains("finite set 'missingUnion' extracted 0 values but minSize is 1"),
        "{findings:?}"
    );
}
