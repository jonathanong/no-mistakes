use super::*;

#[test]
fn standalone_variants_reuse_prepared_source_reads() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-conflict-ordering/variants/one-bad"),
    );
    let query = root.join("src/query.ts");
    let files = [query.clone(), root.join("schema.json")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    check_with_files_and_sources(&root, &config(), &files, &sources).unwrap();
    assert_eq!(sources.physical_read_count(), 2);
}
