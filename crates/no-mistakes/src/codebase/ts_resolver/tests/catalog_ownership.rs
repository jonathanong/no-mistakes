use super::*;

#[test]
fn automatic_catalog_keeps_a_symlinked_config_root_lexical() {
    let fixture = symlinked_root_tsconfig_fixture();
    let root = fixture.join("root");
    let config = root.join("tsconfig.json");
    let entry = root.join("src/entry.ts");
    let linked_value = fixture.join("config/src/value.ts");
    let visible = vec![config.clone(), entry.clone(), linked_value];
    let catalog = TsConfigCatalog::from_visible(&root, std::slice::from_ref(&root), &visible);

    // The config's aliases are relative to its visible symlink location, not
    // to the physical target that happens to contain the config file.
    assert_eq!(catalog.provenance_for(&entry).config, Some(config));
    assert!(catalog.fixed_config().is_none());

    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    assert!(!resolver.uses_fixed_resolver());
    assert_eq!(resolver.resolve("@linked/value", &entry), None);
}

#[test]
fn catalog_keeps_symlink_root_config_paths_lexical_for_extends_and_config_dir() {
    let root = symlink_workspace_tsconfig_fixture();
    let config = root.join("tsconfig.json");
    let base = root.join("tsconfig.base.json");
    let entry = root.join("src/entry.ts");
    let value = root.join("src/value.ts");
    let project_config = root.join("project/tsconfig.json");
    let project_entry = root.join("project/src/entry.ts");
    let project_value = root.join("project/src/value.ts");
    let visible = vec![
        config.clone(),
        base,
        entry.clone(),
        value.clone(),
        project_config.clone(),
        project_entry.clone(),
        project_value.clone(),
    ];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &visible);
    let sources = snapshot.source_store_for(&root);
    let catalog = TsConfigCatalog::from_visible_and_sources(
        &root,
        std::slice::from_ref(&root),
        &visible,
        &sources,
    );

    assert_eq!(catalog.provenance_for(&entry).config, Some(config));
    assert_eq!(catalog.config_for(&entry).dir, root);
    let resolver = ScopedImportResolver::new(
        &catalog,
        &visible.into_iter().collect::<crate::fx::PathSet>(),
    );
    assert_eq!(resolver.resolve("@linked/value", &entry), Some(value));
    assert_eq!(
        catalog.provenance_for(&project_entry).config,
        Some(project_config)
    );
    assert_eq!(
        resolver.resolve("@project/value", &project_entry),
        Some(project_value)
    );
    assert_eq!(sources.physical_read_count(), 3);
}

#[test]
fn source_backed_catalog_reuses_prepared_workspace_facts() {
    let root = workspace_tsconfig_fixture();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let visible = snapshot.paths_for(&root);
    let workspace =
        crate::codebase::workspaces::load_indexed_from_source_store(&root, &sources).unwrap();
    let workspace_manifest_parses = sources.json_parse_count();

    let catalog = TsConfigCatalog::from_visible_and_sources_with_workspace(
        &root,
        std::slice::from_ref(&root),
        &visible,
        &sources,
        &workspace,
    );

    assert_eq!(
        sources.json_parse_count(),
        workspace_manifest_parses,
        "catalog discovery must reuse the prepared workspace instead of reparsing manifests"
    );
    assert_eq!(
        catalog.config_for(&root.join("apps/web/src/entry.ts")).dir,
        root.join("apps/web")
    );
}

#[test]
fn catalog_appends_json_to_dotted_extends_and_reference_basenames() {
    let root = dotted_config_paths_tsconfig_fixture();
    let referenced = root.join("packages/referenced");
    let config = root.join("tsconfig.json");
    let base = root.join("tsconfig.base.json");
    let entry = root.join("src/entry.ts");
    let value = root.join("src/value.ts");
    let referenced_config = referenced.join("tsconfig.build.json");
    let referenced_entry = referenced.join("src/entry.ts");
    let referenced_value = referenced.join("src/value.ts");
    let visible = vec![
        config.clone(),
        base,
        entry.clone(),
        value.clone(),
        referenced_config.clone(),
        referenced_entry.clone(),
        referenced_value.clone(),
    ];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &visible);
    let sources = snapshot.source_store_for(&root);
    let catalog = TsConfigCatalog::from_visible_and_sources(
        &root,
        std::slice::from_ref(&root),
        &visible,
        &sources,
    );

    assert_eq!(catalog.provenance_for(&entry).config, Some(config));
    assert_eq!(
        catalog.provenance_for(&referenced_entry).config,
        Some(referenced_config)
    );
    let resolver = ScopedImportResolver::new(
        &catalog,
        &visible.into_iter().collect::<crate::fx::PathSet>(),
    );
    assert_eq!(resolver.resolve("@root/value", &entry), Some(value));
    assert_eq!(
        resolver.resolve("@referenced/value", &referenced_entry),
        Some(referenced_value)
    );
    assert!(catalog.diagnostics().is_empty());
    assert_eq!(sources.physical_read_count(), 3);
}

#[test]
fn catalog_prefers_an_exact_dotted_non_json_extends_file() {
    let fixture = dotted_config_paths_tsconfig_fixture().join("exact-file");
    let config = fixture.join("tsconfig.json");
    // Keep both files: TypeScript chooses the exact dotted path before trying
    // the appended `.json` fallback.
    let exact_base = fixture.join("tsconfig.base");
    let json_base = fixture.join("tsconfig.base.json");
    let visible = vec![config.clone(), exact_base, json_base];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&fixture, &visible);
    let sources = snapshot.source_store_for(&fixture);
    let mut builder = CatalogBuilder::new(
        &fixture,
        std::slice::from_ref(&fixture),
        &visible,
        Some(&sources),
        None,
    );

    let effective = builder.load_effective(&config).unwrap();

    assert_eq!(effective.tsconfig().paths[0].0, "@exact/*");
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn broken_nested_config_blocks_an_applicable_parent_owner() {
    let root = broken_child_boundary_tsconfig_fixture();
    let nested = root.join("src/feature");
    let root_config = root.join("tsconfig.json");
    let broken_config = nested.join("tsconfig.json");
    let root_entry = root.join("src/root.ts");
    let nested_entry = nested.join("entry.ts");
    let value = root.join("src/value.ts");
    let visible = vec![
        root_config.clone(),
        broken_config,
        root_entry.clone(),
        nested_entry.clone(),
        value,
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), nested], &visible);

    assert_eq!(
        catalog.provenance_for(&root_entry).config,
        Some(root_config)
    );
    assert!(catalog.provenance_for(&nested_entry).config.is_none());
    let resolver = ScopedImportResolver::new(
        &catalog,
        &visible.into_iter().collect::<crate::fx::PathSet>(),
    );
    assert_eq!(resolver.resolve("@root/value", &nested_entry), None);
    assert!(catalog
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.kind == TsConfigDiagnosticKind::InvalidConfig));
}

#[test]
fn symlink_root_broken_child_boundary_blocks_the_canonical_parent_owner() {
    let root = symlink_workspace_tsconfig_fixture();
    let nested = root.join("tests");
    let config = root.join("tsconfig.json");
    let base = root.join("tsconfig.base.json");
    let project_config = root.join("project/tsconfig.json");
    let broken_config = nested.join("tsconfig.json");
    let importer = nested.join("dynamic-manual-mock.test.ts");
    let visible = vec![
        config,
        base,
        project_config,
        broken_config,
        importer.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), nested], &visible);

    assert!(catalog.provenance_for(&importer).config.is_none());
    assert!(catalog
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.kind == TsConfigDiagnosticKind::InvalidConfig));
}

#[test]
fn extended_same_root_tsconfig_base_does_not_compete_for_ownership() {
    let root = extended_base_tsconfig_fixture();
    let entry = root.join("src/entry.ts");
    let value = root.join("src/value.ts");
    let visible = vec![
        root.join("tsconfig.json"),
        root.join("tsconfig.base.json"),
        entry.clone(),
        value.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, std::slice::from_ref(&root), &visible);

    assert_eq!(
        catalog.provenance_for(&entry).config,
        Some(root.join("tsconfig.json"))
    );
    assert!(catalog.diagnostics().is_empty());
    let resolver = ScopedImportResolver::new(
        &catalog,
        &visible.into_iter().collect::<crate::fx::PathSet>(),
    );
    assert_eq!(resolver.resolve("@app/value", &entry), Some(value));
}

#[test]
fn automatic_catalog_prefers_primary_configs_but_keeps_referenced_auxiliaries() {
    let root = auxiliary_ownership_tsconfig_fixture();
    let app = root.join("packages/app");
    let referenced = root.join("packages/referenced");
    let app_entry = app.join("src/entry.ts");
    let primary_value = app.join("src/primary/value.ts");
    let referenced_entry = referenced.join("src/entry.ts");
    let referenced_value = referenced.join("src/runtime/value.ts");
    let app_primary = app.join("tsconfig.json");
    let app_auxiliary = app.join("tsconfig.build.json");
    let referenced_auxiliary = referenced.join("tsconfig.build.json");
    let visible = vec![
        root.join("package.json"),
        app.join("package.json"),
        app_primary.clone(),
        app_auxiliary.clone(),
        app_entry.clone(),
        primary_value.clone(),
        app.join("src/auxiliary/value.ts"),
        referenced.join("package.json"),
        referenced_auxiliary.clone(),
        referenced_entry.clone(),
        referenced_value.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, std::slice::from_ref(&root), &visible);

    // The sibling build config overlaps app sources but is not a project root
    // or reference, so automatic ownership stays with the primary config.
    assert_eq!(catalog.provenance_for(&app_entry).config, Some(app_primary));
    let visible = visible.into_iter().collect::<crate::fx::PathSet>();
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    assert_eq!(
        resolver.resolve("@primary/value", &app_entry),
        Some(primary_value)
    );

    // Project references explicitly opt an auxiliary config into ownership.
    assert_eq!(
        catalog.provenance_for(&referenced_entry).config,
        Some(referenced_auxiliary)
    );
    assert_eq!(
        resolver.resolve("@referenced/value", &referenced_entry),
        Some(referenced_value)
    );
    assert!(catalog.diagnostics().is_empty());

    let auxiliary = load_tsconfig(&app_auxiliary).unwrap();
    let forced = TsConfigCatalog::forced(&root, auxiliary, Some(app_auxiliary));
    let forced_resolver = ScopedImportResolver::new(&forced, &visible);
    assert_eq!(
        forced_resolver.resolve("@auxiliary/value", &app_entry),
        Some(app.join("src/auxiliary/value.ts"))
    );
}

#[test]
fn independently_seeded_config_remains_an_owner_when_another_project_extends_it() {
    let root = workspace_tsconfig_fixture();
    let base = root.join("packages/base-owner");
    let extender = root.join("packages/extender");
    let visible = [
        base.join("package.json"),
        base.join("tsconfig.json"),
        base.join("src/value.ts"),
        extender.join("package.json"),
        extender.join("tsconfig.json"),
        extender.join("src/value.ts"),
    ];
    // Both package roots are independently inferred workspace candidates, so
    // the base remains an owner even though the extender inherits it.
    let catalog = TsConfigCatalog::from_visible(&root, &[base.clone(), extender], &visible);
    assert_eq!(
        catalog.provenance_for(&base.join("src/value.ts")).config,
        Some(base.join("tsconfig.json"))
    );
}

#[test]
fn reference_outside_the_visible_snapshot_is_not_loaded() {
    let root = hidden_reference_tsconfig_fixture();
    let entry = root.join("src/entry.ts");
    let value = root.join("src/value.ts");
    let visible = vec![root.join("tsconfig.json"), entry.clone(), value];
    let catalog = TsConfigCatalog::from_visible(&root, std::slice::from_ref(&root), &visible);

    assert_eq!(
        catalog.provenance_for(&entry).config,
        Some(root.join("tsconfig.json"))
    );
    assert_eq!(
        catalog.diagnostics(),
        vec![TsConfigDiagnostic::config(
            TsConfigDiagnosticKind::InvalidReference,
            &root.join("tsconfig.json"),
            format!(
                "referenced config {} is not in the visible analysis paths",
                root.join("hidden/tsconfig.json").display()
            ),
        )]
    );
    let resolver = ScopedImportResolver::new(
        &catalog,
        &visible.into_iter().collect::<crate::fx::PathSet>(),
    );
    assert_eq!(resolver.resolve("@hidden/value", &entry), None);
}
