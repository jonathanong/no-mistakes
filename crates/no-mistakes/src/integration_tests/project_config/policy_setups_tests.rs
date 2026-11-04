#[test]
fn merged_setup_items_keep_their_individual_fragment_declarations() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/split-provenance/fixture/setup-items");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let dependencies = super::dependencies(&root, &config.tests.vitest.projects["unit"]);
    assert_eq!(dependencies.len(), 2);
    for (dependency, name) in dependencies.iter().zip(["first", "second"]) {
        assert_eq!(
            dependency.declaration_path,
            root.join(format!(".no-mistakes/tests/{name}.yml"))
        );
        assert_eq!(
            dependency.resolved_path.as_ref(),
            Some(&crate::codebase::ts_resolver::normalize_path(
                &root.join(format!("setup/{name}.ts"))
            ))
        );
        assert!(dependency
            .trigger_paths
            .contains(&dependency.declaration_path));
        assert_eq!(dependency.resolution_base, root);
    }
}
