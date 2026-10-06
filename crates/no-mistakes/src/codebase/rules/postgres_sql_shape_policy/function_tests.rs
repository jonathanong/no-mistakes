use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

pub(super) fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture/banned-functions"),
    )
}

fn config(names: &str) -> NoMistakesConfig {
    NoMistakesConfig { rules: vec![RuleDef { rule: RULE_ID.into(), scope: Some(RuleScope::Repository), options: serde_yaml::from_str(&format!("bannedShapes: [banned-function-call]\nshapeOptions:\n  bannedFunctionCall:\n    functions: [{names}]\nimportSpecifier: '@example/db'\ntrustedSqlTags: [{{module: '@example/db', name: sql}}]\nunanalyzableSql: ignore")).unwrap(), ..Default::default() }], ..Default::default() }
}

#[test]
fn functions_are_detected_in_all_expression_positions_and_not_identifiers() {
    let root = fixture();
    let findings = check_with_files(
        &root,
        &config("pg_sleep, pg_sleep_for, pg_sleep_until"),
        &[root.join("positions.sql")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>()
    );
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some(BANNED_FUNCTION_CALL)));
}

#[test]
fn configured_names_preserve_qualifiers_and_quoted_case() {
    let root = fixture();
    for (names, lines) in [
        ("pg_sleep", vec![1, 2, 3]),
        ("public.pg_sleep", vec![2]),
        ("'\"PG_SLEEP\"'", vec![4]),
        ("'\"custom.schema\".\"odd.function\"'", vec![5]),
        ("custom.other", vec![6]),
        ("'custom.\"Other\"'", vec![7]),
    ] {
        let findings = check_with_files(&root, &config(names), &[root.join("names.sql")]).unwrap();
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            lines,
            "{names}: {findings:?}"
        );
    }
}

#[test]
fn parameterized_embedded_calls_reuse_shared_function_facts() {
    let root = fixture();
    let findings = check_with_files(
        &root,
        &config("pg_sleep, pg_sleep_for, pg_sleep_until"),
        &[root.join("embedded.test.ts")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        vec![2, 3, 4, 5, 6]
    );
}

#[test]
fn recoverable_routines_and_execute_retain_function_calls() {
    let root = fixture();
    let findings = check_with_files(
        &root,
        &config("pg_sleep, pg_sleep_for, pg_sleep_until"),
        &[root.join("routines.sql")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        vec![2, 3, 4, 5, 8, 9, 12, 13]
    );
}

#[test]
fn empty_and_blank_function_names_are_configuration_errors_only_when_enabled() {
    for names in ["", "' '", "'pg_catalog.'"] {
        let cfg = config(names);
        let opts: Options = cfg.rules[0].try_rule_options().unwrap();
        assert!(compile_options(&opts).is_err(), "{names}");
    }
    assert!(compile_options(&Options::default()).is_ok());
}

#[test]
fn procedural_conditions_and_wrapped_queries_keep_calls_and_locations() {
    let root = fixture();
    let findings = check_with_files(
        &root,
        &config("pg_sleep, pg_sleep_for, pg_sleep_until"),
        &[root.join("procedural.sql")],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        vec![2, 5, 6, 7, 8, 11]
    );
}

#[test]
fn dynamic_calls_use_the_new_shape_target_and_suppression_is_consistent() {
    let root = fixture();
    let mut cfg = config("pg_sleep");
    cfg.rules[0].options["unanalyzableSql"] = serde_yaml::Value::String("fail".into());
    let findings = check_with_files(&root, &cfg, &[root.join("dynamic.test.ts")]).unwrap();
    assert!(!findings.is_empty());
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some(BANNED_FUNCTION_CALL)));
    let files = [root.join("disabled.sql"), root.join("disabled-file.sql")];
    let mut findings = check_with_files(&root, &cfg, &files).unwrap();
    assert_eq!(findings.len(), 3);
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty());
}
