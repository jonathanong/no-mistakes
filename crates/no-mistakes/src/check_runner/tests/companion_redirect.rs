use std::path::PathBuf;

/// Regression: the aggregate filesystem task runs with the redirect rule
/// filtered out of the config, which used to make the prepared companion
/// selection unmatchable and abort the whole check.
#[test]
fn companion_imports_survive_redirect_rule_config_filtering() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/required-companion-imports/repository-scope-abort");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();

    let results = super::run_all(root, None, None).unwrap();

    let rules: Vec<&str> = results.rules.iter().map(|f| f.rule.as_str()).collect();
    assert!(rules.contains(&"required-companion-imports"), "{rules:?}");
    assert!(rules.contains(&"nextjs-redirect-destinations"), "{rules:?}");
}
