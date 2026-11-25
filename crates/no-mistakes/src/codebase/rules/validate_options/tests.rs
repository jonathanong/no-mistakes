use super::*;
use crate::test_support::rule_fixture_root;

fn saved_config(name: &str) -> NoMistakesConfig {
    let file = rule_fixture_root("options-audit")
        .join("issue-4110")
        .join(name);
    serde_yaml::from_str(&std::fs::read_to_string(file).unwrap()).unwrap()
}

#[test]
fn missing_location_falls_back_to_the_config_file() {
    let config = saved_config("invalid-options.yml");
    let error = validate_rule_options(
        Path::new("project"),
        Path::new("configs/fallback.yml"),
        &config,
        &|_| None,
    )
    .unwrap_err()
    .to_string();
    assert!(error.starts_with("configs/fallback.yml:1:"), "{error}");
    assert!(error.contains("options.bogusOption"), "{error}");
}

#[test]
fn in_memory_rule_options_still_validate_shared_policies() {
    let config = saved_config("legacy-policies.yml");
    let exports = config.rules[0]
        .strict_rule_options::<NoOptions>()
        .err()
        .unwrap();
    let inventory = config.rules[1]
        .try_rule_options::<NoOptions>()
        .err()
        .unwrap();
    assert!(exports.to_string().contains("options.unanalyzableSql"));
    assert!(inventory
        .to_string()
        .contains("does not support options.unanalyzable"));
}

fn source_rule_errors(name: &str) -> (String, String) {
    let root = rule_fixture_root("options-audit").join("issue-4110");
    let config = saved_config(name);
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let swift = crate::codebase::rules::swift_no_unguarded_darwin_import::diagnostics_with_files_and_sources(
        &root, &config, &[], &sources, false,
    )
    .err()
    .unwrap();
    let docs =
        crate::codebase::rules::required_local_docs::check_required_doc_section(&root, &config)
            .unwrap_err();
    (format!("{swift:#}"), format!("{docs:#}"))
}

#[test]
fn in_memory_source_rule_calls_report_invalid_options() {
    let (swift, docs) = source_rule_errors("semantic-errors.yml");
    assert!(swift.contains("options.allow"), "{swift}");
    assert!(docs.contains("options.glob"), "{docs}");
}

#[test]
fn in_memory_source_rules_report_invalid_application_scope() {
    let (swift, docs) = source_rule_errors("legacy-policies.yml");
    assert!(swift.contains("include contains invalid glob"), "{swift}");
    assert!(docs.contains("include contains invalid glob"), "{docs}");
}

#[test]
fn in_memory_source_rules_reject_malformed_option_types() {
    let (swift, docs) = source_rule_errors("malformed-source-options.yml");
    assert!(swift.contains("at options.allow: invalid type"), "{swift}");
    assert!(docs.contains("at options.glob: invalid type"), "{docs}");
}

#[test]
fn every_registered_configurable_rule_id_has_a_unique_option_validator() {
    let mut validated = std::collections::BTreeSet::new();
    for id in [
        crate::codebase::rules::GITHUB_ACTIONS_ACTION_INPUT_POLICY,
        crate::codebase::rules::GITHUB_ACTIONS_CACHE_KEY_POLICY,
        crate::codebase::rules::GITHUB_ACTIONS_NO_REQUIRED_PR_PATH_FILTERS,
        crate::codebase::rules::QUEUE_RATE_LIMIT_ERROR_DELAY,
        crate::codebase::rules::VITEST_DEFAULT_EXCLUDES,
        crate::codebase::rules::POSTGRES_LOCK_REASON,
    ]
    .into_iter()
    .chain(
        VALIDATORS
            .iter()
            .flat_map(|group| group.iter().map(|(id, _)| *id)),
    ) {
        assert!(
            validated.insert(id),
            "duplicate option validator registered for `{id}`"
        );
    }
    let missing: Vec<_> = crate::codebase::rules::rule_ids::configurable_rule_ids()
        .filter(|id| !validated.contains(id))
        .collect();
    assert!(
        missing.is_empty(),
        "registered rules missing from VALIDATORS: {missing:?}"
    );
}

#[test]
fn missing_validator_returns_an_internal_error_instead_of_no_options_fallback() {
    let rule: RuleDef = serde_yaml::from_str("rule: missing-validator-rule").unwrap();
    let error = application_errors(Path::new("."), &NoMistakesConfig::default(), &rule)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains(
            "internal error: rule `missing-validator-rule` has no registered option validator"
        ),
        "{error}"
    );
}

#[test]
fn strict_rule_options_handles_non_struct_targets_and_enum_error_paths() {
    #[derive(serde::Deserialize)]
    enum Mode {
        Variant { count: usize },
    }
    #[derive(Default, serde::Deserialize)]
    struct EnumOptions {
        mode: Option<Mode>,
    }

    let raw_rule: RuleDef = serde_yaml::from_str("rule: example\noptions:\n  flag: true").unwrap();
    let value: serde_yaml::Value = raw_rule.strict_rule_options().unwrap();
    assert_eq!(value["flag"].as_bool(), Some(true));

    let ok_rule: RuleDef =
        serde_yaml::from_str("rule: example\noptions:\n  mode: !Variant\n    count: 1").unwrap();
    let ok: EnumOptions = ok_rule.strict_rule_options().unwrap();
    assert!(matches!(ok.mode, Some(Mode::Variant { count: 1 })));

    let bad_enum_rule: RuleDef = serde_yaml::from_str(
        "rule: example\nname: enum-case\noptions:\n  mode: !Variant\n    count: bad",
    )
    .unwrap();
    let error = bad_enum_rule
        .strict_rule_options::<EnumOptions>()
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("application `enum-case`"), "{error}");
    assert!(error.contains("options.mode.Variant.count"), "{error}");
}

#[test]
fn push_application_errors_without_origin_or_fallback_keeps_raw_message() {
    let mut out = Vec::new();
    push_application_errors(
        Path::new("."),
        &NoMistakesConfig::default(),
        0,
        None,
        BTreeSet::from(["raw application error".to_string()]),
        &mut out,
    );
    assert_eq!(out, vec!["raw application error"]);
}
