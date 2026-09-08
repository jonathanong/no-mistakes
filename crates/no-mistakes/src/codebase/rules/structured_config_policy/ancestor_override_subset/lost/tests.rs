use super::*;
use std::path::PathBuf;

#[test]
fn effective_rules_ignores_children_on_a_different_path_root() {
    let value: Value = serde_yaml::from_str("rules: { x: error }").unwrap();
    let mapping = value.as_mapping().unwrap();
    let rules = mapping_value(mapping, "rules")
        .and_then(Value::as_mapping)
        .unwrap();
    let files = compile_value_globs(
        &serde_yaml::from_str::<Value>("files: '**/*.ts'")
            .unwrap()
            .as_mapping()
            .unwrap()
            .clone(),
        "files",
    )
    .unwrap();
    let overrides = [Override {
        rules,
        files,
        exclude_files: None,
        dir: Path::new("/repo"),
    }];

    let effective = effective_rules(&overrides, Path::new("relative/file.ts"), |override_| {
        override_.dir
    });

    assert!(effective.is_empty());
}

#[test]
fn collect_overrides_rejects_invalid_globs_before_skipping_empty_rules() {
    let value: Value =
        serde_yaml::from_str("overrides:\n  - files: ['[']\n    rules: {}\n").unwrap();
    let assertion = ValueAssertion::default();
    let keys = Keys::from_assertion(&assertion);
    let ancestors = [Ancestor {
        path: PathBuf::from("/repo/base.json"),
        rel: "base.json".to_string(),
        value,
    }];

    let (_, findings) = collect_overrides(&ancestors, "nested/.oxlintrc.json", &assertion, &keys);

    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("files must contain valid string globs"));
}
