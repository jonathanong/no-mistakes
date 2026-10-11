use super::*;

#[test]
fn standalone_variants_keep_one_parse_and_one_read_per_source() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-lock-ordering/variants/one-bad"),
    );
    let query = root.join("src/query.ts");
    let files = [query.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    crate::ast::begin_parse_count(&root);
    let findings =
        check_with_files_and_sources(&root, &default_config(), &files, &sources).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&query), Some(&1), "{counts:#?}");
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(!findings[0]
        .target
        .as_deref()
        .unwrap()
        .contains("unanalyzable"));
}
