use super::*;

#[test]
fn inferred_project_roots_reuse_inventory_and_keep_request_roots_distinct() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check/unused-allow-missing/fixture"),
    );
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("implicit-root.yml"))).unwrap();
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[
        root.join("next.config.mjs"),
        root.join("vite.config.ts"),
    ])))
    .with_complete_inventory();
    for _ in 0..2 {
        assert_eq!(
            sources
                .inferred_roots_for_projects(&root, config.projects.values())
                .nextjs,
            Some(Some(root.clone()))
        );
    }
    assert_eq!(sources.inferred_roots.get().unwrap().len(), 1);
    assert_eq!(
        sources
            .inferred_roots_for_projects(&root.join("src"), config.projects.values())
            .nextjs,
        Some(None)
    );
    assert_eq!(sources.inferred_roots.get().unwrap().len(), 2);
    config.projects.get_mut("web").unwrap().root = Some("src".into());
    assert_eq!(
        sources
            .inferred_roots_for_projects(&root, config.projects.values())
            .nextjs,
        None
    );
    assert_eq!(sources.inferred_roots.get().unwrap().len(), 2);
    assert_eq!(sources.physical_read_count(), 0);
    config.projects.get_mut("web").unwrap().root = None;
    config.projects.get_mut("web").unwrap().type_ = Some(ProjectType::Vitejs);
    assert_eq!(
        sources
            .inferred_roots_for_projects(&root, config.projects.values())
            .vitejs,
        Some(Some(root.clone()))
    );
    assert_eq!(sources.physical_read_count(), 1);
    config.projects.get_mut("web").unwrap().type_ = Some(ProjectType::Remix);
    assert_eq!(
        sources
            .inferred_roots_for_projects(&root, config.projects.values())
            .remix,
        Some(None)
    );
    assert_eq!(sources.physical_read_count(), 1);
    let direct = SourceStore::new(Arc::new(FileInventory::from_paths(&[
        root.join("remix.config.js")
    ])));
    assert_eq!(
        direct
            .inferred_roots_for_projects(&root, config.projects.values())
            .remix,
        Some(Some(root.clone()))
    );
    assert_eq!(direct.physical_read_count(), 0);
    let shared = SourceStore::new(Arc::new(FileInventory::from_paths(&[
        root.join("remix-vite/vite.config.ts")
    ])));
    assert_eq!(
        shared
            .inferred_roots_for_projects(&root, config.projects.values())
            .remix,
        Some(Some(root.join("remix-vite")))
    );
    config.projects.get_mut("web").unwrap().type_ = Some(ProjectType::Vitejs);
    assert_eq!(
        shared
            .inferred_roots_for_projects(&root, config.projects.values())
            .vitejs,
        Some(None)
    );
    assert_eq!(shared.physical_read_count(), 1);
}
