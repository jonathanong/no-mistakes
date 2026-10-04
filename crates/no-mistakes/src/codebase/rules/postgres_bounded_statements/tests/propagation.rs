use super::offenders;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::{path::PathBuf, sync::Arc};

#[test]
fn prepared_keys_preserve_reversed_chains_and_unseeded_cycles() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/postgres-bounds");
    for size in [16, 64, 256] {
        let name = format!("catalog-{size}.json");
        let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[root.join(&name)])));
        let catalog = SchemaCatalog::load(&root, &name, &sources).unwrap();
        for kind in ["reversed", "cycle"] {
            let sql = std::fs::read_to_string(root.join(format!("{kind}-{size}.sql"))).unwrap();
            let facts = extract_sql_statement_facts(&sql);
            let found: Vec<_> = facts
                .bounds
                .iter()
                .flat_map(|fact| offenders(fact, &catalog))
                .collect();
            assert_eq!(
                found.len(),
                if kind == "cycle" { size } else { 0 },
                "{kind}/{size}"
            );
        }
    }
}
