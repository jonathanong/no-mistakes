use super::validate;
use std::path::{Path, PathBuf};

fn load(name: &str) -> (String, PathBuf) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/rule-keys/fixture")
        .join(name);
    (std::fs::read_to_string(&path).unwrap(), path)
}

fn check(name: &str) -> anyhow::Result<()> {
    let (source, path) = load(name);
    validate(&source, &path)
}

#[test]
fn shapes_other_than_a_rule_list_are_left_to_the_main_parser() {
    for name in [
        "no-rules.yml",
        "rules-not-list.yml",
        "entry-not-mapping.yml",
        "syntax-error.yml",
        "top-level-list.yml",
        "valid.yml",
    ] {
        check(name).unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

#[test]
fn json_configs_report_the_unknown_key() {
    let error = check("unknown.json").unwrap_err().to_string();
    assert!(error.contains("unknown key `serverity`"), "{error}");
    assert!(error.contains("rule `unique-exports`"), "{error}");
}

#[test]
fn non_string_keys_are_rejected_with_a_debug_name() {
    let error = check("numeric-key.yml").unwrap_err().to_string();
    assert!(error.contains("unknown key `Number(1)`"), "{error}");
}

#[test]
fn supported_keys_match_the_serialized_rule_entry() {
    let mut rule = crate::config::v2::schema::RuleDef {
        severity: crate::config::v2::schema::RuleSeverity::Advisory,
        ..Default::default()
    };
    rule.rule = "x".into();
    let value = serde_yaml::to_value(&rule).unwrap();
    let mut keys: Vec<_> = value
        .as_mapping()
        .unwrap()
        .keys()
        .map(|key| key.as_str().unwrap().to_string())
        .collect();
    let mut supported: Vec<_> = crate::config::v2::schema::RuleDef::SUPPORTED_KEYS
        .iter()
        .map(|key| key.to_string())
        .collect();
    keys.sort();
    supported.sort();
    assert_eq!(keys, supported);
}
