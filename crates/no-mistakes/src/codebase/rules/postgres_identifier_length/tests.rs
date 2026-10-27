use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

mod followups;

fn unit(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres/identifier-length")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn omitted_options_default_to_63_bytes_and_every_sql_file() {
    let opts: Options = serde_yaml::from_str("{}").unwrap();
    assert_eq!(opts.max_bytes, 63);
    let compiled = compile_options(&opts).unwrap();
    assert_eq!(compiled.max_bytes, 63);
    assert_eq!(
        compiled.schema.sql_include,
        crate::codebase::postgres::PostgresSchemaOptions::default().sql_include
    );
}

#[test]
fn locked_empty_sql_include_does_not_expand_to_every_sql_file() {
    let opts: Options = serde_yaml::from_str("sqlIncludeLocked: true").unwrap();
    assert!(compile_options(&opts)
        .unwrap()
        .schema
        .sql_include
        .is_empty());
}

#[test]
fn max_bytes_outside_1_through_63_is_a_config_error() {
    for value in [0, 64, -1] {
        let Err(error) = compile_options(&Options {
            max_bytes: value,
            ..Options::default()
        }) else {
            panic!("maxBytes {value} should be rejected");
        };
        let error = error.to_string();
        assert!(
            error.contains("postgres-identifier-length option maxBytes: must be from 1 through 63"),
            "{error}"
        );
    }
}

#[test]
fn invalid_sql_include_is_a_config_error() {
    let path = unit("max-bytes.sql");
    let Err(error) = check_with_files(
        path.parent().unwrap(),
        &config("sqlInclude: [\"[\"]\nmaxBytes: 63\n"),
        std::slice::from_ref(&path),
    ) else {
        panic!("invalid sqlInclude should be rejected");
    };
    let error = error.to_string();
    assert!(
        error.contains("postgres-identifier-length option sqlInclude:"),
        "{error}"
    );
}

#[test]
fn a_short_name_is_left_unchanged() {
    assert_eq!(scan::truncate_identifier("id", 63), "id");
}
