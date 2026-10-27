use super::super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

// Output cases moved to cli_postgres_snapshot_sql_shape_policy.

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// (line, target) for every finding.
pub(super) fn found(name: &str, yaml: &str, file: &str) -> Vec<(usize, String)> {
    let root = fixture(name);
    let mut found: Vec<_> = check_with_files(&root, &config(yaml), &[root.join(file)])
        .unwrap()
        .into_iter()
        .map(|finding| (finding.line, finding.target.unwrap()))
        .collect();
    found.sort();
    found
}

pub(super) fn at(findings: &[(usize, &str)]) -> Vec<(usize, String)> {
    findings
        .iter()
        .map(|(line, target)| (*line, target.to_string()))
        .collect()
}

#[test]
fn an_implicit_fetch_is_reported_at_the_fetch_clause() {
    // `FETCH FIRST ROW ONLY` writes no count, so the finding points at the FETCH keyword (a
    // line-level suppression beside it applies), not at the start of the query.
    let none = "sqlInclude: ['sql/**/*.sql']\nbannedShapes: [literal-limit]\nshapeOptions:\n  \
                literalLimit:\n    allowedValues: []\n";
    assert_eq!(
        found("fail-bounded-iteration", none, "sql/002.sql"),
        at(&[(3, "literal-limit"), (6, "literal-limit")])
    );
}

#[test]
fn unanalyzable_sql_names_an_enabled_iteration_shape() {
    let shapes = |literal, sweep| BannedShapes {
        literal_limit: literal,
        keyset_only_sweep: sweep,
        ..Default::default()
    };
    assert_eq!(shapes(true, true).unanalyzable_target(), "literal-limit");
    assert_eq!(
        shapes(false, true).unanalyzable_target(),
        "keyset-only-sweep"
    );
}

#[test]
fn shape_options_are_validated() {
    let error = |yaml: &str| {
        compile_options(&serde_yaml::from_str::<Options>(yaml).unwrap())
            .err()
            .unwrap()
            .to_string()
    };
    let negative = error("shapeOptions: {literalLimit: {allowedValues: [1, -2]}}");
    assert!(
        negative.contains("allowedValues: negative value -2"),
        "{negative}"
    );
    let predicate = error("shapeOptions: {keysetOnlySweep: {nonSelectivePredicates: [' ']}}");
    assert!(
        predicate.contains("nonSelectivePredicates: empty string"),
        "{predicate}"
    );
    let table = error("shapeOptions: {keysetOnlySweep: {ignoreTables: ['']}}");
    assert!(table.contains("ignoreTables: empty string"), "{table}");
    assert!(compile_options(&Options::default()).is_ok());
    // An unparseable predicate is still compared textually.
    assert!(compile_options(
        &serde_yaml::from_str::<Options>(
            "shapeOptions: {keysetOnlySweep: {nonSelectivePredicates: ['(((']}}"
        )
        .unwrap()
    )
    .is_ok());
    let _ = Path::new("");
}
