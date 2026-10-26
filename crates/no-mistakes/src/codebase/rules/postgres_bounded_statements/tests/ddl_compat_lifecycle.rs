use super::{extract_sql_statement_facts, offenders, SchemaCatalog};
use crate::codebase::postgres::statements::SqlBoundItemKind;

#[test]
fn metadata_compatibility_preserves_temporary_relations_and_prepared_plans() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-compat/lifecycle.sql"
    ));
    let catalog = SchemaCatalog::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-compat/lifecycle-catalog.json"
    )))
    .unwrap();
    let control = extract_sql_statement_facts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/ddl-compat/lifecycle-control.sql"
    )));
    assert_eq!(control.bounds.len(), 2);
    assert!(control
        .bounds
        .iter()
        .all(|bound| !offenders(bound, &catalog).is_empty()));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 7);
    // The catalog deliberately has permanent namesakes. Losing temporary state
    // or the prepared definition would expose an unbounded physical relation.
    for bound in &facts.bounds {
        assert!(offenders(bound, &catalog).is_empty(), "{bound:#?}");
    }
    for index in [1, 3, 4, 5, 6] {
        let items = &facts.bounds[index].query.items;
        assert_eq!(items.len(), 1);
        assert!(
            matches!(items[0].kind, SqlBoundItemKind::Opaque),
            "{index}: {items:#?}"
        );
    }
    // The initial query and the prepared definition keep their original limits.
    // EXECUTE replays the destination effect; the final read proves it survived.
    for index in [0, 2] {
        let query = &facts.bounds[index].query;
        assert!(query.capped);
        assert!(matches!(&query.items[0].kind, SqlBoundItemKind::Table(name) if name == "orders"));
    }
}
