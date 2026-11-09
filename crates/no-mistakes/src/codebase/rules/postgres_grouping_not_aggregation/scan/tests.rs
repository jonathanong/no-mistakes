use super::*;
use crate::codebase::postgres::{prepared::PreparationError, SchemaCatalog};
use std::sync::Arc;

#[test]
fn prepared_read_errors_are_not_silenced_except_missing_raw_sources() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/postgres-facts/grouping/fixture");
    let path = root.join("missing-source.sql");
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let opts = compile::options(&Options::default()).unwrap();
    let catalog = SchemaCatalog::default();
    let mut facts = CheckFactMap::default();
    assert!(scan_file(&root, &opts, &path, &sources, &facts, &catalog).is_err());
    for (kind, skipped) in [
        (std::io::ErrorKind::NotFound, true),
        (std::io::ErrorKind::InvalidData, false),
    ] {
        facts.postgres.statements.insert(
            path.clone(),
            Err(Arc::new(PreparationError {
                message: Arc::from("saved source cannot be read"),
                source_kind: Some(kind),
            })),
        );
        assert_eq!(
            scan_file(&root, &opts, &path, &sources, &facts, &catalog).is_ok(),
            skipped
        );
    }
    facts
        .postgres
        .statements
        .insert(path.clone(), Ok(Arc::new(Vec::new())));
    assert!(scan_file(&root, &opts, &path, &sources, &facts, &catalog).is_err());
    let embedded = root.join("exact-origins.ts");
    assert!(scan_file(&root, &opts, &embedded, &sources, &facts, &catalog).is_err());
}
