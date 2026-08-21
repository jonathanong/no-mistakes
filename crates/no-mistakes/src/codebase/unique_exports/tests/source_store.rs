use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::sync::Arc;

#[test]
fn nextjs_package_lookup_reuses_prepared_source_store() {
    let root = fixture("unique-exports-nextjs");
    let manifest = root.join("package.json");
    let page = root.join("web/app/users/page.tsx");
    let inventory = Arc::new(FileInventory::from_paths(&[manifest.clone(), page.clone()]));
    let store = SourceStore::new(inventory);

    let lookup = scan::NextJsProjectLookup::with_sources(
        &root,
        std::slice::from_ref(&page),
        &[manifest.clone(), page.clone()],
        Some(&store),
    );
    assert!(lookup.contains_file(&page));
    let reads = store.physical_read_count();
    assert!(
        reads >= 1,
        "Next.js package.json detection must read through the request SourceStore"
    );

    let lookup = scan::NextJsProjectLookup::with_sources(
        &root,
        std::slice::from_ref(&page),
        &[manifest.clone(), page.clone()],
        Some(&store),
    );
    assert!(lookup.contains_file(&page));
    assert_eq!(
        store.physical_read_count(),
        reads,
        "Next.js package.json detection must reuse the request SourceStore"
    );
}

#[test]
fn collect_source_files_from_facts_reports_missing_fact_shapes() {
    let root = fixture("unique-exports-basic");
    let file = root.join("src/a.ts");
    let files = vec![file.clone()];
    let missing = crate::codebase::check_facts::CheckFactMap::default();

    assert!(
        scan::test_support::collect_source_files_from_facts(&root, &files, &missing, false)
            .unwrap_err()
            .to_string()
            .contains("missing shared facts")
    );

    let mut parse_error = crate::codebase::check_facts::CheckFactMap::default();
    parse_error.ts.insert(
        file.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("export const Broken =".into()),
            parse_error: Some("bad syntax".to_string()),
            ..Default::default()
        }
        .into(),
    );
    assert!(scan::test_support::collect_source_files_from_facts(
        &root,
        &files,
        &parse_error,
        false
    )
    .unwrap_err()
    .to_string()
    .contains("bad syntax"));

    let mut missing_source = crate::codebase::check_facts::CheckFactMap::default();
    missing_source.ts.insert(file.clone(), Default::default());
    assert!(scan::test_support::collect_source_files_from_facts(
        &root,
        &files,
        &missing_source,
        false
    )
    .unwrap_err()
    .to_string()
    .contains("missing source facts"));

    let mut missing_symbols = crate::codebase::check_facts::CheckFactMap::default();
    missing_symbols.ts.insert(
        file,
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("export const value = 1;".into()),
            ..Default::default()
        }
        .into(),
    );
    assert!(scan::test_support::collect_source_files_from_facts(
        &root,
        &files,
        &missing_symbols,
        false
    )
    .unwrap_err()
    .to_string()
    .contains("missing symbol facts"));
}
