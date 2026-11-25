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
