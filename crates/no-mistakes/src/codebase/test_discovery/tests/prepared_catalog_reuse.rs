use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn empty_catalog(root: &Path) -> std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog> {
    std::sync::Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        root,
        std::slice::from_ref(&root.to_path_buf()),
        &[],
    ))
}

#[test]
fn unchanged_roots_reuse_the_preliminary_catalog() {
    let root = PathBuf::from("/repo");
    let preliminary = empty_catalog(&root);
    let mut final_roots = vec![root.join("."), root.clone()];
    let rebuilds = AtomicUsize::new(0);

    let final_catalog = reuse_or_rebuild_prepared_catalog(
        std::sync::Arc::clone(&preliminary),
        std::slice::from_ref(&root),
        &mut final_roots,
        |_| {
            rebuilds.fetch_add(1, Ordering::Relaxed);
            empty_catalog(&root)
        },
    );

    assert!(std::sync::Arc::ptr_eq(&preliminary, &final_catalog));
    assert_eq!(rebuilds.load(Ordering::Relaxed), 0);
}

#[test]
fn added_roots_rebuild_an_automatic_catalog_once() {
    let root = PathBuf::from("/repo");
    let preliminary = empty_catalog(&root);
    let mut final_roots = vec![root.clone(), root.join("packages/app")];
    let rebuilds = AtomicUsize::new(0);

    let final_catalog = reuse_or_rebuild_prepared_catalog(
        std::sync::Arc::clone(&preliminary),
        std::slice::from_ref(&root),
        &mut final_roots,
        |_| {
            rebuilds.fetch_add(1, Ordering::Relaxed);
            empty_catalog(&root)
        },
    );

    assert!(!std::sync::Arc::ptr_eq(&preliminary, &final_catalog));
    assert_eq!(rebuilds.load(Ordering::Relaxed), 1);
}

#[test]
fn forced_catalog_ignores_added_candidate_roots() {
    let root = PathBuf::from("/repo");
    let preliminary = std::sync::Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::forced(
        &root,
        crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths: Vec::new(),
            paths_dir: root.clone(),
            base_url: None,
        },
        Some(root.join("tsconfig.json")),
    ));
    let mut final_roots = vec![root.clone(), root.join("packages/app")];

    let final_catalog = reuse_or_rebuild_prepared_catalog(
        std::sync::Arc::clone(&preliminary),
        std::slice::from_ref(&root),
        &mut final_roots,
        |_| panic!("forced catalogs must never rebuild for candidate roots"),
    );

    assert!(std::sync::Arc::ptr_eq(&preliminary, &final_catalog));
}
