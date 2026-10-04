use no_mistakes::codebase::postgres::{
    extract_embedded_sql_from_source, extract_sql_statement_facts,
    extract_sql_statement_facts_for_embedded_call, EmbeddedSqlOptions, SqlCursorBound,
    SqlValueForm,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

#[test]
fn public_embedded_call_composition_preserves_placeholder_provenance() {
    let path = fixture("public-statement-facts.ts");
    let source = std::fs::read_to_string(&path).expect("embedded SQL fixture");
    let embedded = extract_embedded_sql_from_source(&path, &source, &EmbeddedSqlOptions::default());

    let recovered = &embedded.calls[0];
    assert_eq!(recovered.recovered_placeholder_positions.len(), 2);
    let facts = extract_sql_statement_facts_for_embedded_call(recovered)
        .expect("the call has recovered SQL");

    let sweep = &facts.sweeps[0];
    assert_eq!(sweep.table, "orders");
    assert_eq!(sweep.order_columns, ["id"]);
    let fake_marker_column = sweep
        .conjuncts
        .iter()
        .find(|conjunct| conjunct.text.contains("fake_id"))
        .expect("the user-authored marker-shaped identifier remains a predicate");
    assert!(fake_marker_column.cursor_columns.is_empty());

    let cursor = sweep
        .conjuncts
        .iter()
        .find(|conjunct| !conjunct.cursor_columns.is_empty())
        .expect("the interpolated cursor bind is recognized");
    assert_eq!(cursor.cursor_columns, ["id"]);
    assert_eq!(cursor.cursor_bound, Some(SqlCursorBound::Lower));

    assert!(extract_sql_statement_facts_for_embedded_call(&embedded.calls[1]).is_none());
}

#[test]
fn embedded_placeholder_positions_are_authoritative_across_fact_families() {
    let path = fixture("provenance-across-facts.ts");
    let source = std::fs::read_to_string(&path).expect("embedded SQL fixture");
    let embedded = extract_embedded_sql_from_source(&path, &source, &EmbeddedSqlOptions::default());

    let query = &embedded.calls[0];
    assert_eq!(query.recovered_placeholder_positions.len(), 1);
    let query_facts = extract_sql_statement_facts_for_embedded_call(query).unwrap();
    let order = query_facts.bounds[0]
        .query
        .items
        .iter()
        .find(|item| matches!(&item.kind, no_mistakes::codebase::postgres::SqlBoundItemKind::Table(table) if table == "orders"))
        .unwrap();
    assert_eq!(
        order
            .pins
            .iter()
            .map(|pin| pin.column.as_str())
            .collect::<Vec<_>>(),
        ["tenant_id"]
    );
    assert!(!query_facts.bounds[0].query.capped);
    let relation = query_facts.selects[0]
        .relations
        .iter()
        .find(|relation| relation.table == "orders")
        .unwrap();
    assert_eq!(relation.constrained_columns, ["tenant_id"]);

    // Standalone extraction has no source provenance and retains its legacy marker heuristic.
    let standalone = extract_sql_statement_facts(query.sql_text.as_deref().unwrap());
    let standalone_order = &standalone.bounds[0].query.items[0];
    assert_eq!(
        standalone_order
            .pins
            .iter()
            .map(|pin| pin.column.as_str())
            .collect::<Vec<_>>(),
        ["id", "tenant_id"]
    );
    assert!(standalone.bounds[0].query.capped);

    // Some(empty) still means embedded provenance is known: quoted and unquoted user markers
    // cannot become binds merely because their names resemble generated placeholders.
    let quoted = &embedded.calls[1];
    assert!(quoted.recovered_placeholder_positions.is_empty());
    let quoted_facts = extract_sql_statement_facts_for_embedded_call(quoted).unwrap();
    assert!(quoted_facts.bounds[0].query.items[0].pins.is_empty());
    assert!(!quoted_facts.bounds[0].query.capped);

    let insert = &embedded.calls[2];
    let insert_facts = extract_sql_statement_facts_for_embedded_call(insert).unwrap();
    let insert = &insert_facts.inserts[0];
    assert_eq!(insert.assignments[0].form, SqlValueForm::Placeholder);
    assert_eq!(
        insert.assignments[1].form,
        SqlValueForm::SelfRef {
            column: "sql_placeholder_2".into()
        }
    );
    assert_eq!(insert.assignments[2].form, SqlValueForm::Placeholder);
    let conflict = insert.on_conflict.as_ref().unwrap();
    assert_eq!(
        conflict.assignments[0].form,
        SqlValueForm::SelfRef {
            column: "sql_placeholder_2".into()
        }
    );
    assert_eq!(conflict.assignments[1].form, SqlValueForm::Placeholder);

    let fake_exists = extract_sql_statement_facts_for_embedded_call(&embedded.calls[3]).unwrap();
    let fake_exists_facts = fake_exists
        .selects
        .iter()
        .flat_map(|select| &select.exists_set_operations)
        .collect::<Vec<_>>();
    assert_eq!(fake_exists_facts.len(), 1);
    assert!(!fake_exists_facts[0].restricted);
    let real_exists = extract_sql_statement_facts_for_embedded_call(&embedded.calls[4]).unwrap();
    assert_eq!(
        real_exists
            .selects
            .iter()
            .flat_map(|select| &select.exists_set_operations)
            .filter(|exists| exists.restricted)
            .count(),
        1
    );

    let generated_limit =
        extract_sql_statement_facts_for_embedded_call(&embedded.calls[5]).unwrap();
    assert!(generated_limit.bounds[0].query.capped);

    let mutation_exists =
        extract_sql_statement_facts_for_embedded_call(&embedded.calls[6]).unwrap();
    let mutation_exists_facts = mutation_exists
        .selects
        .iter()
        .flat_map(|select| &select.exists_set_operations)
        .collect::<Vec<_>>();
    assert_eq!(mutation_exists_facts.len(), 1);
    assert!(!mutation_exists_facts[0].restricted);

    let fake_conflict = extract_sql_statement_facts_for_embedded_call(&embedded.calls[7]).unwrap();
    let proof = &fake_conflict.inserts[0]
        .on_conflict
        .as_ref()
        .unwrap()
        .where_proof;
    assert!(proof.distinct_from_excluded.is_empty());
}
