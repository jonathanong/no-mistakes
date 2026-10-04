use super::fixture_root;

#[test]
fn table_identity_is_independent_of_from_and_join_predicate_visit_order() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/table-source-association.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let root = fixture_root();
    let path = root.join("schema-table-source-association.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog = crate::codebase::postgres::SchemaCatalog::load(
        &root,
        "schema-table-source-association.json",
        &sources,
    )
    .unwrap();
    let names: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect();
    // Outer page caps retain their existing semantics; the standalone physical
    // control must still be reported by a catalog that actually contains ids.
    assert_eq!(names, ["ids"]);
}

#[test]
fn table_bound_identity_retains_the_original_physical_arm_lines() {
    let sql =
        std::fs::read_to_string(fixture_root().join("sql/table-source-association.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let mut lines = std::collections::BTreeSet::new();
    for bound in &facts.bounds {
        physical_ids(&bound.query, &mut lines);
    }
    assert_eq!(lines.into_iter().collect::<Vec<_>>(), [6, 13, 19, 26, 30]);
}

fn physical_ids(
    query: &crate::codebase::postgres::SqlBoundQuery,
    lines: &mut std::collections::BTreeSet<usize>,
) {
    use crate::codebase::postgres::{SqlBoundItemKind, SqlPinSource};
    for item in &query.items {
        match &item.kind {
            SqlBoundItemKind::Table(table) if table == "ids" => {
                lines.insert(item.line);
            }
            SqlBoundItemKind::Query(query) => physical_ids(query, lines),
            _ => {}
        }
        for pin in &item.pins {
            if let SqlPinSource::Query(query) | SqlPinSource::ReadQuery(query) = &pin.source {
                physical_ids(query, lines);
            }
        }
    }
}
