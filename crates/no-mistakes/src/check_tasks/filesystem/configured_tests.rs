use no_mistakes::config::v2::schema::{NoMistakesConfig, RuleDef, RuleScope};

#[test]
fn vite_optimize_deps_nested_imports_enables_the_filesystem_check() {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: no_mistakes::codebase::rules::vite_optimize_deps_nested_imports::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        ..RuleDef::default()
    });
    assert!(super::super::filesystem_rules_configured(&config));
}
