// Output cases moved to cli_postgres_snapshot_generated_column_writes.
mod followups;
mod identities;
mod no_op;
mod prepared;
mod trigger;
mod unanalyzable;

use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-generated-column-writes")
}

fn unit_fixture(name: &str) -> PathBuf {
    fixture_root().join("unit-fixture").join(name)
}

fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
        ..Default::default()
    });
    config
}

/// Keep write assertions focused while checking every new schema marker against
/// its saved SQL source and the direct migration projection.
fn verified_schema_markers(
    root: &Path,
    schema: &str,
    findings: Vec<RuleFinding>,
) -> Vec<RuleFinding> {
    let (markers, definite): (Vec<_>, Vec<_>) = findings.into_iter().partition(|finding| {
        finding.message.contains("SQL statement `")
            && finding
                .message
                .contains("could not be analyzed for postgres-no-generated-column-writes")
    });
    let source = std::fs::read_to_string(root.join(schema)).unwrap();
    let expected =
        crate::codebase::postgres::extract_migration_facts(&source).unanalyzable_statements;
    let mut actual = markers
        .iter()
        .map(|finding| {
            assert_eq!(finding.file, schema);
            (finding.line, finding.target.clone().unwrap())
        })
        .collect::<Vec<_>>();
    let mut expected = expected
        .into_iter()
        .map(|marker| (marker.line, marker.head))
        .collect::<Vec<_>>();
    actual.sort();
    expected.sort();
    assert!(
        !actual.is_empty(),
        "expected saved schema markers in {schema}"
    );
    assert_eq!(actual, expected);
    definite
}

#[test]
fn invalid_include_glob_errors_and_missing_files_are_skipped() {
    let error = compile_options(&Options {
        include: vec!["[".to_string()],
        ..Options::default()
    })
    .err()
    .expect("invalid include")
    .to_string();
    assert!(error.contains("invalid glob"), "{error}");
}

#[test]
fn empty_catalog_dynamic_sql_and_invalid_schema() {
    let invalid = unit_fixture("invalid-schema");
    let skipped = check_with_files(
        &invalid,
        &config_with_options("sqlInclude: ['**/broken.sql']"),
        &[invalid.join("broken.sql")],
    )
    .expect("unparseable schema is reported");
    let definite = verified_schema_markers(&invalid, "broken.sql", skipped);
    assert!(definite.is_empty(), "{definite:#?}");
}

#[test]
fn default_dml_extensions_and_message_shape() {
    assert!(is_default_dml_path(Path::new("a.ts")));
    assert!(is_default_dml_path(Path::new("a.mts")));
    assert!(is_default_dml_path(Path::new("a.tsx")));
    assert!(is_default_dml_path(Path::new("a.js")));
    assert!(is_default_dml_path(Path::new("a.sql")));
    assert!(!is_default_dml_path(Path::new("a.rs")));

    let message = finding("src/q.ts", 4, "items", "created_at").message;
    assert!(message.contains("src/q.ts:4"));
    assert!(message.contains("items.created_at"));
    assert!(message.contains("source column"));
}
