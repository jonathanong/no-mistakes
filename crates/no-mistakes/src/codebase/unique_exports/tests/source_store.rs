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
