fn findings(name: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    let base = crate::test_support::rule_fixture_root("forbidden-calls");
    let base = base.parent().unwrap();
    crate::codebase::rules::run_check(
        &base.join("mock-boundaries/fixture"),
        Some(&base.join(format!("configs/mock-boundaries-{name}.yml"))),
        None,
    )
    .unwrap()
}

#[test]
fn automocks_keep_module_scope_initializer_calls() {
    let findings = findings("initializer");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].file.ends_with("src/initializer-module.mts"));
}

#[test]
fn automocks_keep_imported_dependency_initializer_calls() {
    let findings = findings("initializer-dependency");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].file.ends_with("src/initializer-helper.mts"));
}

#[test]
fn full_factory_replacement_skips_the_real_initializer() {
    assert!(findings("initializer-factory").is_empty());
}
