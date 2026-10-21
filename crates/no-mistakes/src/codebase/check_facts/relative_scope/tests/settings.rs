use super::*;

#[test]
fn missing_config_falls_back_without_inventing_a_package() {
    let root = PathBuf::from("/repo");
    let config = or_empty_tsconfig(&root, None);
    assert_eq!(config.dir, root);
    assert!(config.paths.is_empty());
    let present = Arc::new(TsConfig {
        dir: root.clone(),
        ..TsConfig::default()
    });
    assert!(Arc::ptr_eq(
        &or_empty_tsconfig(&root, Some(Arc::clone(&present))),
        &present
    ));
    let workspace = workspace_or_empty(Err(anyhow::anyhow!("missing manifest")));
    assert!(workspace.package_by_name("@example/db").is_none());
}

#[test]
fn package_roots_are_cached_only_for_named_workspace_packages() {
    let root = fixture_root();
    let sources = sources_for(&root);
    let workspace = Arc::new(load_indexed_from_source_store(&root, &sources).expect("workspace"));
    let mut cache = HashMap::new();
    let named = workspace
        .package_by_name("@example/db")
        .expect("db")
        .dir
        .clone();
    assert_eq!(
        remember_package_root(&mut cache, "@example/db", &workspace, Some(named.clone()))
            .as_deref(),
        Some(named.as_path())
    );
    assert_eq!(cache.get("@example/db"), Some(&named));
    assert!(
        remember_package_root(&mut cache, "@missing/pkg", &workspace, Some(named.clone()))
            .is_some()
    );
    assert!(!cache.contains_key("@missing/pkg"));
    assert!(remember_package_root(&mut cache, "../transaction", &workspace, Some(named)).is_some());
    assert!(!cache.contains_key("../transaction"));
    assert!(remember_package_root(&mut cache, "@example/db", &workspace, None).is_none());
}
