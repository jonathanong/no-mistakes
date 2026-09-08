use super::*;

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
