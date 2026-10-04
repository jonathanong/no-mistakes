use super::evaluate_postgres_bounds;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::{path::PathBuf, sync::Arc};

#[test]
fn prepared_postgres_adapter_consumes_saved_statement_facts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-bounds");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[
        root.join("catalog-16.json")
    ])));
    let catalog = SchemaCatalog::load(&root, "catalog-16.json", &sources).unwrap();
    for (name, expected) in [("reversed-16.sql", 0), ("cycle-16.sql", 16)] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        assert_eq!(evaluate_postgres_bounds(&facts, &catalog), expected);
    }
}
