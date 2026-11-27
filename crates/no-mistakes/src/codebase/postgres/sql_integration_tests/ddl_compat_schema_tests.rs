#[test]
fn schema_projection_has_no_false_uncertainty_and_retains_index_facts() {
    use crate::codebase::postgres::extract_schema_facts;
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::{path::PathBuf, sync::Arc};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/source/ddl-compat");
    let paths = [root.join("valid.sql"), root.join("after-do.sql")];
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&paths)));
    let facts = extract_schema_facts(&root, &sources, &paths).expect("schema projection");
    for file in facts {
        assert!(
            file.unanalyzable_statements.is_empty(),
            "{:?}: {:?}",
            file.path,
            file.unanalyzable_statements
        );
        assert_eq!(file.indexes.len(), 1);
    }
}
