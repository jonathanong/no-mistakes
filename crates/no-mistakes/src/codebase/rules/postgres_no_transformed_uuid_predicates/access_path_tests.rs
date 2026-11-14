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
            assert_eq!(
                columns.judge(&uses).status,
                expected,
                "{name}:{line}: {uses:?}"
            );
        }
    }
}
