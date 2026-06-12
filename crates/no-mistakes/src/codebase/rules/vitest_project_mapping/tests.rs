use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-project-mapping")
            .join(name),
    )
}

fn load_config(root: &Path) -> NoMistakesConfig {
    let mut config =
        crate::config::v2::load_v2_config(root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("{}").unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn reports_unmapped_and_ambiguous_vitest_tests() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![
        root.join("src/a.test.ts"),
        root.join("src/shared.test.ts"),
        root.join("src/unmapped.test.ts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].file, "src/shared.test.ts");
    assert!(findings[0].message.contains("multiple Vitest projects"));
    assert_eq!(findings[1].file, "src/unmapped.test.ts");
    assert!(findings[1].message.contains("does not map"));
}

#[test]
fn default_extensions_include_spec_files() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![root.join("src/unmapped.spec.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.spec.ts");
}

#[test]
fn default_extensions_include_javascript_test_files() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![root.join("src/unmapped.test.js")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.test.js");
}

#[test]
fn default_extensions_include_commonjs_typescript_test_files() {
    let root = fixture_root("fixture");
    let config = load_config(&root);
    let files = vec![root.join("src/unmapped.test.cts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.test.cts");
}

#[test]
fn scopes_can_limit_checked_test_candidates() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.rules[0].options = serde_yaml::from_str("scopes: [src/a.test.ts]\n").unwrap();
    let files = vec![
        root.join("src/a.test.ts"),
        root.join("src/shared.test.ts"),
        root.join("src/unmapped.test.ts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn root_scope_matches_all_relative_test_paths() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.rules[0].options = serde_yaml::from_str("scopes: [/]\n").unwrap();
    let files = vec![root.join("src/unmapped.test.ts")];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.test.ts");
}

#[test]
fn configured_projects_and_custom_extensions_are_checked() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.configs = None;
    config.tests.vitest.projects.insert(
        "custom".to_string(),
        serde_yaml::from_str(
            r#"
include: [src/custom.spec.ts]
"#,
        )
        .unwrap(),
    );
    config.rules[0].options =
        serde_yaml::from_str("testExtensions: [.spec.ts]\nscopes: [src]\n").unwrap();
    let files = vec![
        root.join("src/custom.spec.ts"),
        root.join("src/unmapped.spec.ts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "src/unmapped.spec.ts");
}

#[test]
fn missing_config_paths_surface_load_errors() {
    let root = fixture_root("fixture");
    let mut config = load_config(&root);
    config.tests.vitest.configs = Some(serde_yaml::from_str("missing.config.ts").unwrap());
    let result = check_with_files(&root, &config, &[root.join("src/a.test.ts")]);

    assert!(result.is_err());
}
