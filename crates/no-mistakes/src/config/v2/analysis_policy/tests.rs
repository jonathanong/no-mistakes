use super::*;
use std::path::PathBuf;

fn config(name: &str) -> NoMistakesConfig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/rule-analysis-policy/fixture");
    super::super::load_v2_config(&root, Some(PathBuf::from(name).as_path())).unwrap()
}

#[test]
fn analysis_precedence_and_sql_alias_preserve_existing_payloads() {
    let mut config = config("precedence.yml");
    assert_eq!(
        config.requested_rule_analysis(&config.rules[0]).unwrap(),
        EffectiveRuleAnalysis {
            effort: Effort::Low,
            unanalyzable: Unanalyzable::Advise,
        }
    );
    assert_eq!(
        config.rules[0].options["unanalyzableSql"].as_str(),
        Some("advise")
    );
    assert_eq!(
        config.rules[1].options["unanalyzableSql"].as_str(),
        Some("report")
    );
    assert_eq!(
        config.rules[2].options["unanalyzableSql"].as_str(),
        Some("ignore")
    );
    config.effort_override = Some(Effort::Max);
    assert_eq!(
        config
            .requested_rule_analysis(&config.rules[0])
            .unwrap()
            .effort,
        Effort::Max
    );
    assert_eq!(
        config
            .effective_rule_analysis(&config.rules[0])
            .unwrap()
            .effort,
        Effort::Max
    );
    assert_eq!(
        config
            .requested_rule_analysis(&config.rules[3])
            .unwrap()
            .unanalyzable,
        Unanalyzable::Report
    );
}

#[test]
fn default_analysis_adds_no_serialized_fields_or_options() {
    let mut config = config("default.yml");
    let before = serde_json::to_value(&config).unwrap();
    config.materialize_analysis_defaults().unwrap();
    assert_eq!(serde_json::to_value(&config).unwrap(), before);
    assert!(before.get("effort").is_none());
    assert!(before.get("unanalyzable").is_none());
    assert_eq!(
        config.requested_rule_analysis(&config.rules[0]).unwrap(),
        EffectiveRuleAnalysis::default()
    );
}

#[test]
fn shared_options_do_not_leak_into_rule_specific_deserializers() {
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct Empty {}
    let config = config("precedence.yml");
    config.rules[3].try_rule_options::<Empty>().unwrap();
}

#[test]
fn canonical_report_maps_to_each_legacy_sql_parser_vocabulary() {
    let mut config = config("precedence.yml");
    config.unanalyzable = Some(Unanalyzable::Report);
    config.rules[2].options = serde_yaml::Value::Null;
    config.materialize_analysis_defaults().unwrap();
    assert_eq!(
        config.rules[2].options["unanalyzableSql"].as_str(),
        Some("fail")
    );
    config.rules[1].options = serde_yaml::Value::Null;
    config.materialize_analysis_defaults().unwrap();
    assert_eq!(
        config.rules[1].options["unanalyzableSql"].as_str(),
        Some("report")
    );
}

#[test]
fn non_sql_common_alias_is_equivalent_and_removed_before_specific_options() {
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct Empty {}
    let common = config("common-alias.yml");
    for rule in &common.rules {
        rule.try_rule_options::<Empty>().unwrap();
    }
    assert_eq!(
        common.requested_rule_analysis(&common.rules[0]).unwrap(),
        common.requested_rule_analysis(&common.rules[1]).unwrap()
    );
    assert_eq!(
        common.requested_rule_analysis(&common.rules[0]).unwrap(),
        common.requested_rule_analysis(&common.rules[2]).unwrap()
    );
    let invalid = config("invalid-alias.yml");
    assert!(validate_low_alias(&invalid.rules[0]).is_err());
}

#[test]
fn common_only_options_preserve_rule_options_default_without_serde_defaults() {
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct Required {
        required_field: String,
    }
    let common = config("common-alias.yml");
    for rule in &common.rules {
        assert!(rule
            .try_rule_options::<Required>()
            .unwrap()
            .required_field
            .is_empty());
    }
    let empty = config("default.yml");
    assert!(empty.rules[0]
        .try_rule_options::<Required>()
        .unwrap()
        .required_field
        .is_empty());
    let effort = config("precedence.yml");
    assert!(effort.rules[3]
        .try_rule_options::<Required>()
        .unwrap()
        .required_field
        .is_empty());
}

#[test]
fn shared_defaults_leave_malformed_rule_options_for_the_original_parser() {
    for name in ["malformed-options.yml", "malformed-sequence.yml"] {
        let mut malformed = config(name);
        let options = malformed.rules[0].options.clone();
        malformed.materialize_analysis_defaults().unwrap();
        assert_eq!(malformed.rules[0].options, options);
    }
}

#[test]
fn inherited_ignored_postgres_alias_payloads_remain_opaque() {
    #[derive(Deserialize, Default)]
    struct IgnoredOptions {}
    let inherited = config("project-fail.yml");
    for rule in &inherited.rules {
        assert_eq!(rule.options["unanalyzableSql"].as_str(), Some("fail"));
        rule.try_rule_options::<IgnoredOptions>().unwrap();
    }
    let opaque = config("opaque-legacy.yml");
    for rule in &opaque.rules {
        rule.try_rule_options::<IgnoredOptions>().unwrap();
        assert_eq!(
            opaque.requested_rule_analysis(rule).unwrap().unanalyzable,
            Unanalyzable::Report
        );
    }
}
