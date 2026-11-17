use super::*;

fn mixed_config(name: &str) -> NoMistakesConfig {
    let path = fixture("mixed-ddl-trigger")
        .join("../../configs")
        .join(name);
    serde_yaml::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn complete_catalog_proves_ddl_and_trigger_writes_together() {
    let options = mixed_config("mixed-ddl-trigger-complete.yml");
    let findings = check(&fixture("mixed-ddl-trigger"), &options).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn ordering_catalog_proves_ddl_without_inventing_trigger_writes() {
    let options = mixed_config("mixed-ddl-trigger-ordering.yml");
    let findings = check(&fixture("mixed-ddl-trigger"), &options).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 11);
    assert_eq!(findings[0].target.as_deref(), Some(CHECKS[0]));
}

#[test]
fn ignored_checks_do_not_consume_an_invalid_catalog() {
    let mut options = mixed_config("mixed-ddl-trigger-complete.yml");
    options.rules[0].options["schemaCatalogPath"] = "missing.json".into();
    options.rules[0].options["checks"] =
        serde_yaml::to_value(BTreeMap::from(CHECKS.map(|check| (check, "ignore")))).unwrap();
    let findings = check(&fixture("mixed-ddl-trigger"), &options).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn non_source_selection_does_not_consume_an_invalid_catalog() {
    let root = fixture("mixed-ddl-trigger");
    let mut options = mixed_config("mixed-ddl-trigger-complete.yml");
    options.rules[0].options["schemaCatalogPath"] = "missing.json".into();
    let findings = check_with_files(&root, &options, &[root.join("schema-complete.json")]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
