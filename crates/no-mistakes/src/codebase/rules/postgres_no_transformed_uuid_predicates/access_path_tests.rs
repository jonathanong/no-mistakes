use super::evaluate::Columns;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog, SqlVerdictStatus};
use std::collections::BTreeMap;

#[test]
fn expression_indexes_and_outer_join_residuals_preserve_driving_controls() {
    let root = super::tests::fixture().join("access-paths");
    let sources = super::super::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let columns = Columns::new(&catalog);
    for (name, expected) in [
        ("expression-pass.sql", SqlVerdictStatus::ProvenClear),
        ("expression-positive.sql", SqlVerdictStatus::Violation),
        ("outer-residual.sql", SqlVerdictStatus::Abstain),
        ("outer-positive.sql", SqlVerdictStatus::Violation),
    ] {
        let source = std::fs::read_to_string(root.join(name)).unwrap();
        let facts = extract_sql_statement_facts(&source);
        assert!(!facts.parse_failed, "{name}");
        let mut lines = BTreeMap::<_, Vec<_>>::new();
        for use_ in facts.transformed_column_uses {
            lines.entry(use_.line).or_default().push(use_);
        }
        assert_eq!(lines.len(), source.lines().count(), "{name}");
        for (line, uses) in lines {
            for use_ in &uses {
                if let Some(first) = uses
                    .iter()
                    .find(|other| other.expression_site == use_.expression_site)
                {
                    assert!(std::sync::Arc::ptr_eq(&first.expression, &use_.expression));
                    assert!(std::sync::Arc::ptr_eq(
                        &first.expression_relations,
                        &use_.expression_relations
                    ));
                }
            }
            assert_eq!(
                columns.judge(&uses).status,
                expected,
                "{name}:{line}: {uses:?}"
            );
        }
    }
}

#[test]
fn grouped_outer_joins_keep_residual_uncertainty_and_driving_findings() {
    let root = super::tests::fixture().join("access-paths");
    let sources = super::super::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let columns = Columns::new(&catalog);
    let source = std::fs::read_to_string(root.join("outer-grouped.sql")).unwrap();
    let facts = extract_sql_statement_facts(&source);
    assert!(!facts.parse_failed, "{facts:#?}");
    let mut lines = BTreeMap::<_, Vec<_>>::new();
    for use_ in facts.transformed_column_uses {
        lines.entry(use_.line).or_default().push(use_);
    }
    let expected = [
        (1, SqlVerdictStatus::Abstain, true),
        (2, SqlVerdictStatus::Abstain, true),
        (3, SqlVerdictStatus::Abstain, true),
        (4, SqlVerdictStatus::Abstain, true),
        (5, SqlVerdictStatus::Abstain, true),
        (6, SqlVerdictStatus::Abstain, true),
        (7, SqlVerdictStatus::Abstain, true),
        (8, SqlVerdictStatus::Abstain, true),
        (9, SqlVerdictStatus::Violation, false),
        (10, SqlVerdictStatus::Violation, false),
        (11, SqlVerdictStatus::Violation, false),
        (12, SqlVerdictStatus::Violation, false),
        (13, SqlVerdictStatus::Violation, true),
        (14, SqlVerdictStatus::Violation, false),
        (15, SqlVerdictStatus::Abstain, false),
        (16, SqlVerdictStatus::Violation, false),
        (17, SqlVerdictStatus::Violation, false),
        (18, SqlVerdictStatus::Violation, false),
    ];
    assert_eq!(lines.len(), expected.len(), "{lines:?}");
    for (line, status, residual) in expected {
        let uses = lines.get(&line).unwrap_or_else(|| panic!("line {line}"));
        let sql = source.lines().nth(line - 1).unwrap_or("");
        assert_eq!(
            columns.judge(uses).status,
            status,
            "line {line}: {sql}: {uses:?}"
        );
        assert_eq!(
            uses.iter().any(|use_| use_.outer_join_residual),
            residual,
            "line {line}: {sql}"
        );
        for use_ in uses.iter().filter(|use_| use_.outer_join_residual) {
            assert_eq!(
                columns.resolve(use_).unwrap_err(),
                "outer-join-residual-filter",
                "line {line}: {sql}"
            );
        }
        if line == 15 {
            assert!(uses
                .iter()
                .all(|use_| { columns.resolve(use_).unwrap_err() == "unresolved-column-owner" }));
        }
    }
}
