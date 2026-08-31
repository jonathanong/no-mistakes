use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-test-git-sha/fixture")
        .join(name)
}

#[test]
fn reports_selected_full_sha_and_honors_allowed_context() {
    let root = fixture("fail");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = vec![root.join("tests/example.test.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1);
}

#[test]
fn standard_include_selects_only_configured_paths() {
    let root = fixture("pass");
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files = vec![root.join("src/value.ts")];
    assert!(check_with_files(&root, &config, &files).unwrap().is_empty());
}

#[test]
fn selected_source_without_a_sha_is_clean() {
    let root = fixture("pass");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        include: vec!["tests/**/*.test.ts".to_string()],
        ..Default::default()
    });
    let file = root.join("tests/no-sha.test.ts");
    assert!(check_with_files(&root, &config, &[file])
        .unwrap()
        .is_empty());
}

#[test]
fn invalid_allowed_context_regex_is_a_configuration_error() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("allowedContexts: ['(']").unwrap(),
        ..Default::default()
    });
    let root = fixture("fail");
    let error =
        check_with_files(&root, &config, &[root.join("tests/example.test.ts")]).unwrap_err();
    assert!(error.to_string().contains("allowedContexts"), "{error:#}");
}

#[test]
fn standard_exclude_removes_a_selected_test_path() {
    let root = fixture("fail");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        include: vec!["tests/**/*.test.ts".to_string()],
        exclude: vec!["tests/example.test.ts".to_string()],
        ..Default::default()
    });
    assert!(
        check_with_files(&root, &config, &[root.join("tests/example.test.ts")])
            .unwrap()
            .is_empty()
    );
}
