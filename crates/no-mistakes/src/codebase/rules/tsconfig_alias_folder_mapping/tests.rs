use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/tsconfig-alias-folder-mapping/fixture")
        .join(path)
}

fn config_from_yaml(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn agents_config() -> &'static str {
    "tsconfig: tsconfig.json\nbaseDir: backend\nmappings:\n  - prefix: \"@agents\"\n    root: agents\n"
}

// Saved-fixture output cases moved to cli_pkgcfg_snapshot_tsconfig_alias_folder_mapping.
// Config load rejects include `[` before the rule can skip a missing tsconfig,
// so check JSON cannot show this empty result. Extending a missing file makes
// the CLI exit before it prints JSON, so that finding stays in-process.

#[test]
fn existing_unloadable_tsconfig_is_a_finding() {
    let findings = check(
        &fixture("extends-missing"),
        &config_from_yaml(agents_config()),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].file, "tsconfig.json");
    assert!(findings[0]
        .message
        .contains("tsconfig could not be loaded and alias mappings were not checked"));
}

#[test]
fn missing_tsconfig_skips_invalid_rule_path_filters() {
    let mut config =
        config_from_yaml(&agents_config().replace("tsconfig.json", "nonexistent.json"));
    // Preserve the no-op before path filter compilation, including its error precedence.
    config.rules[0].include = vec!["[".to_string()];
    assert!(check(&fixture("pass"), &config).unwrap().is_empty());
}
