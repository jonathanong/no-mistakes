use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn long(label: &str) -> String {
    format!("{label}{}", "x".repeat(64 - label.len()))
}

fn unit(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres/identifier-length")
            .join(name),
    )
}

fn scenario(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-identifier-length/fixture")
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

fn run(path: &Path, yaml: &str) -> Vec<RuleFinding> {
    let root = path.parent().unwrap();
    check_with_files(root, &config(yaml), &[path.to_path_buf()]).unwrap()
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
fn pass_fixture_accepts_a_63_byte_name() {
    let root = scenario("pass");
    let findings = check_with_files(
        &root,
        &config("sqlInclude: [\"migrations/**/*.sql\"]\n"),
        &[root.join("migrations/ok.sql")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn fail_fixture_flags_every_declared_name_and_not_references() {
    let root = scenario("fail");
    let findings = check_with_files(
        &root,
        &config("sqlInclude: [\"migrations/**/*.sql\"]\n"),
        &[root.join("migrations/long.sql")],
    )
    .unwrap();
    let targets: BTreeSet<_> = findings
        .iter()
        .filter_map(|finding| finding.target.clone())
        .collect();
    let expected = [
        "column_name",
        "inline_constraint",
        "table_constraint",
        "table_name",
        "unique_index",
        "trigger_name",
        "function_name",
        "replaced_function",
        "procedure_name",
        "replaced_procedure",
        "view_name",
        "matview_name",
        "enum_type",
        "added_constraint",
        "added_column",
        "renamed_table",
        "renamed_column",
        "renamed_constraint",
        "renamed_index",
        "do_index",
    ]
    .into_iter()
    .map(long)
    .chain(["idx_invoice_line_items_account_id_created_at_status_currency_code".to_string()])
    .collect::<BTreeSet<_>>();
    assert_eq!(targets, expected, "{findings:?}");
    assert!(!targets.contains(&long("referenced_only")));
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.target.as_deref() == Some(long("do_index").as_str()))
            .count(),
        1
    );
    let index = findings
        .iter()
        .find(|finding| {
            finding.target.as_deref()
                == Some("idx_invoice_line_items_account_id_created_at_status_currency_code")
        })
        .unwrap();
    assert_eq!(
        index.message,
        format!(
            "{}:{}: identifier \"idx_invoice_line_items_account_id_created_at_status_currency_code\" is 65 bytes; PostgreSQL truncates identifiers longer than 63 bytes to \"idx_invoice_line_items_account_id_created_at_status_currency_co\"",
            index.file, index.line
        )
    );
}

#[test]
fn multibyte_cut_stays_on_a_char_boundary() {
    let findings = run(&unit("multibyte.sql"), "maxBytes: 63\n");
    assert_eq!(findings.len(), 2, "{findings:?}");
    let euro = "a".repeat(61) + "€";
    let accent = "b".repeat(62) + "é";
    assert!(
        findings.iter().any(|finding| finding.message.contains(&format!(
            "identifier \"{euro}\" is 64 bytes; PostgreSQL truncates identifiers longer than 63 bytes to \"{}\"",
            "a".repeat(61)
        ))),
        "{findings:?}"
    );
    assert!(
        findings.iter().any(|finding| finding.message.contains(&format!(
            "identifier \"{accent}\" is 64 bytes; PostgreSQL truncates identifiers longer than 63 bytes to \"{}\"",
            "b".repeat(62)
        ))),
        "{findings:?}"
    );
}

#[test]
fn quoted_identifier_unescapes_doubled_quotes() {
    let findings = run(&unit("quotes.sql"), "maxBytes: 63\n");
    assert_eq!(findings.len(), 1, "{findings:?}");
    let name = format!("order\"{}", "q".repeat(58));
    assert_eq!(name.len(), 64);
    assert_eq!(findings[0].target.as_deref(), Some(name.as_str()));
}

#[test]
fn the_same_declared_name_is_reported_twice() {
    let findings = run(&unit("duplicate.sql"), "maxBytes: 63\n");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings[0].target, findings[1].target);
    assert_ne!(findings[0].line, findings[1].line);
}

#[test]
fn referenced_names_are_not_findings() {
    assert!(run(&unit("reference.sql"), "maxBytes: 63\n").is_empty());
}

#[test]
fn schema_qualifier_is_not_part_of_the_name() {
    let findings = run(&unit("schema-qualified.sql"), "maxBytes: 63\n");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some(long("qualified_index").as_str())
    );
    assert!(!findings[0].message.contains("app."));
}

#[test]
fn do_block_identifier_uses_the_outer_statement_line() {
    let findings = run(&unit("do-block.sql"), "maxBytes: 63\n");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 4, "{findings:?}");
}

#[test]
fn function_body_identifier_is_remapped_onto_the_outer_file() {
    let findings = run(&unit("function-body.sql"), "maxBytes: 63\n");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some(long("do_index").as_str())
    );
    assert_eq!(findings[0].line, 3, "{findings:?}");
}

#[test]
fn max_bytes_option_changes_the_limit() {
    let findings = run(&unit("max-bytes.sql"), "maxBytes: 10\n");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains(
            "identifier \"abcdefghijk\" is 11 bytes; PostgreSQL truncates identifiers longer than 10 bytes to \"abcdefghij\""
        ),
        "{findings:?}"
    );
    assert!(run(&unit("max-bytes.sql"), "maxBytes: 11\n").is_empty());
}

#[test]
fn sql_include_can_skip_a_migration_file() {
    let root = scenario("fail");
    let findings = check_with_files(
        &root,
        &config("sqlInclude: [\"nope/**/*.sql\"]\n"),
        &[root.join("migrations/long.sql")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn a_short_name_is_left_unchanged() {
    assert_eq!(scan::truncate_identifier("id", 63), "id");
}

#[test]
fn embedded_typescript_sql_is_out_of_scope() {
    assert!(run(
        &unit("ignored.ts"),
        "sqlInclude: [\"**/*.sql\"]\nmaxBytes: 63\n"
    )
    .is_empty());
}
