use super::{fixture_root, offenders, SchemaCatalog};
use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn referential_actions_do_not_expand_the_direct_statement_scope() {
    let root = fixture_root();
    let files = [
        root.join("referential-actions.json"),
        root.join("sql/referential-actions.sql"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let catalog = SchemaCatalog::load(&root, "referential-actions.json", &sources).unwrap();
    for (table, parent, action) in [
        ("orders", "accounts", "cascade"),
        ("order_lines", "orders", "cascade"),
        ("profiles", "accounts", "set null"),
        ("labels", "accounts", "no action"),
    ] {
        let key = &catalog.table(table).unwrap().foreign_keys[0];
        assert_eq!(key.referenced_table, parent);
        assert_eq!(key.on_delete, action);
        assert_eq!(key.on_update, action);
    }
    let sql = sources.read_path(&files[1]).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    // Parent pins stay bounded even with CASCADE, SET NULL, NO ACTION and a
    // two-level cascade in the catalog. This deliberately judges explicit rows.
    assert_eq!(facts.bounds.len(), 4);
    for fact in &facts.bounds[..3] {
        assert!(offenders(fact, &catalog).is_empty());
    }
    let names: Vec<_> = offenders(&facts.bounds[3], &catalog)
        .into_iter()
        .map(|offender| offender.table)
        .collect();
    assert_eq!(names, ["accounts"]);
}
