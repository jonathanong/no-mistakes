// no-mistakes-disable-file rust-max-lines-per-file: legacy resolver coverage suite
use super::*;
use std::collections::HashSet;
use tempfile::TempDir;

impl ImportResolutionCache {
    pub(crate) fn classification_count(&self) -> usize {
        self.classifications
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn request_count(&self) -> usize {
        self.requests.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[test]
fn import_resolution_cache_clear_removes_raw_and_classified_entries() {
    let root = workspace_tsconfig_fixture();
    let importer = root.join("apps/web/src/entry.ts");
    let target = root.join("apps/web/src/runtime/value.ts");
    let cache = ImportResolutionCache::default();
    let key = ResolveKey {
        importing_file: importer,
        specifier: "@runtime/value".to_string(),
    };
    cache.raw_entries.insert(key.clone(), Some(target.clone()));
    cache.final_entries.insert(
        key.clone(),
        ImportClassification {
            resolver_target: Some(target),
            workspace_target: None,
            workspace_recognized: false,
        },
    );

    cache.clear();

    assert!(cache.raw_entries.get(&key).is_none());
    assert!(cache.final_entries.get(&key).is_none());
}

fn write(path: &Path, content: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

fn make_tsconfig(dir: &Path, paths_json: &str) -> TsConfig {
    let content = format!(r#"{{"compilerOptions": {{"paths": {}}}}}"#, paths_json);
    let p = dir.join("tsconfig.json");
    write(&p, &content);
    load_tsconfig(&p).unwrap()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/ts-resolver/fixture")
        .join(name)
}

fn workspace_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/workspace-resolution"),
    )
}

fn scoped_resolution_candidates_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/scoped-resolution-candidates"),
    )
}

fn fixed_root_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/fixed-root-fast-path/root"),
    )
}

fn absolute_glob_rules_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/absolute-glob-rules"),
    )
}

fn exact_include_files_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/exact-include-files"),
    )
}

fn include_config_directory_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/include-config-directory"),
    )
}

fn symlinked_root_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlinked-root-tsconfig"),
    )
}

fn symlink_workspace_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    )
}

fn broken_child_boundary_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/broken-child-boundary"),
    )
}

fn trailing_directory_include_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/trailing-directory-include"),
    )
}

fn dotted_config_paths_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/dotted-config-paths"),
    )
}

fn extended_base_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/extended-base-ownership"),
    )
}

fn hidden_reference_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/hidden-reference"),
    )
}

fn auxiliary_ownership_tsconfig_fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/auxiliary-ownership"),
    )
}

#[test]
fn scoped_catalog_selects_and_resolves_workspace_aliases() {
    let root = workspace_tsconfig_fixture();
    let web = root.join("apps/web");
    let visible = [
        root.join("tsconfig.json"),
        root.join("tsconfig.base.json"),
        web.join("tsconfig.json"),
        web.join("src/entry.ts"),
        web.join("src/runtime/value.ts"),
        root.join("packages/shared/tsconfig.json"),
        root.join("packages/shared/src/message.ts"),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), web.clone()], &visible);
    let importer = web.join("src/entry.ts");
    assert_eq!(
        catalog.provenance_for(&importer).config.as_deref(),
        Some(web.join("tsconfig.json").as_path())
    );
    let visible = visible.into_iter().collect();
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    assert_eq!(
        resolver.resolve("@runtime/value", &importer),
        Some(web.join("src/runtime/value.ts"))
    );
    assert_eq!(
        resolver.resolve("@shared/message", &importer),
        Some(root.join("packages/shared/src/message.ts"))
    );
}

#[test]
fn scoped_resolution_candidates_use_importer_workspace_aliases_for_deleted_targets() {
    let root = scoped_resolution_candidates_fixture();
    let web = root.join("apps/web");
    let importer = web.join("src/entry.ts");
    let visible = [
        root.join("tsconfig.json"),
        web.join("tsconfig.json"),
        importer.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), web.clone()], &visible);
    let visible = visible.into_iter().collect();
    let resolver = ScopedImportResolver::new(&catalog, &visible);

    let candidates = resolver.resolution_candidates("@fixture/deleted", &importer);

    assert!(candidates.contains(&web.join("src/workspace/deleted.ts")));
    assert!(!candidates.contains(&root.join("src/root/deleted.ts")));
}

#[test]
fn scoped_catalog_session_reuses_dynamic_importer_scope_cache() {
    let root = workspace_tsconfig_fixture();
    let web = root.join("apps/web");
    let importer = web.join("src/entry.ts");
    let target = web.join("src/runtime/value.ts");
    let visible = [
        root.join("tsconfig.json"),
        root.join("tsconfig.base.json"),
        web.join("tsconfig.json"),
        importer.clone(),
        target.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, &[root.clone(), web], &visible);
    let visible = visible.into_iter().collect();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));

    let first = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
    assert_eq!(
        first.resolve("@runtime/value", &importer),
        Some(target.clone())
    );
    assert_eq!(first.scope_key_build_count(), 1);
    assert_eq!(first.scope_cache_lookup_count(), 1);
    let first_work = observer.snapshot().work;
    let second = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
    assert_eq!(second.resolve("@runtime/value", &importer), Some(target));
    let repeated_work = observer.snapshot().work;

    assert_eq!(
        repeated_work["resolver.computations"],
        first_work["resolver.computations"]
    );
    assert!(
        repeated_work["resolver.cache_hits"]
            > first_work
                .get("resolver.cache_hits")
                .copied()
                .unwrap_or_default()
    );
}

#[test]
fn queue_compatibility_never_clears_the_standard_fixed_session_cache() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/tsconfig-paths/fixture");
    let importer = root.join("enqueue.ts");
    let target = root.join("queues/email.ts");
    let visible_paths = [importer.clone(), target.clone()];
    let visible = visible_paths.iter().cloned().collect::<HashSet<_>>();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let catalog = TsConfigCatalog::forced(&root, tsconfig, None);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));

    let standard = ScopedImportResolver::new_in_session(&catalog, &visible, &session);
    assert_eq!(standard.resolve("queues/email", &importer), None);
    let before = observer.snapshot().work;
    let queue = ScopedImportResolver::new_in_session(&catalog, &visible, &session)
        .with_queue_compatibility(&root);
    assert_eq!(
        queue.resolve("queues/email", &importer),
        Some(normalize_path(&target))
    );
    assert_eq!(standard.resolve("queues/email", &importer), None);
    let after = observer.snapshot().work;
    assert_eq!(
        after["resolver.computations"],
        before["resolver.computations"] + 1
    );
    assert!(
        after
            .get("resolver.cache_hits")
            .copied()
            .unwrap_or_default()
            > before
                .get("resolver.cache_hits")
                .copied()
                .unwrap_or_default()
    );
}

#[test]
fn standalone_scoped_resolver_builds_one_visibility_scope_per_selected_catalog_config() {
    let root = workspace_tsconfig_fixture();
    let web = root.join("apps/web");
    let worker = root.join("services/worker");
    let web_target = web.join("src/runtime/value.ts");
    let worker_target = worker.join("src/runtime/value.ts");
    let mut visible = [
        root.join("tsconfig.json"),
        root.join("tsconfig.base.json"),
        web.join("tsconfig.json"),
        web_target.clone(),
        worker.join("tsconfig.worker.json"),
        worker_target.clone(),
    ]
    .into_iter()
    .collect::<HashSet<_>>();
    // The resolver must not sort this request-wide universe for each of the
    // synthetic importers below. They model a large one-config package.
    visible.extend((0..256).map(|index| root.join(format!("generated/{index}.ts"))));
    let catalog = TsConfigCatalog::from_visible(
        &root,
        &[root.clone(), web.clone(), worker.clone()],
        &visible.iter().cloned().collect::<Vec<_>>(),
    );
    let resolver = ScopedImportResolver::new(&catalog, &visible);

    for index in 0..64 {
        let importer = web.join(format!("src/generated-{index}.ts"));
        assert_eq!(
            resolver.resolve("@runtime/value", &importer),
            Some(web_target.clone())
        );
    }
    assert_eq!(resolver.scope_key_build_count(), 1);
    assert_eq!(resolver.scope_cache_lookup_count(), 1);
    assert_eq!(resolver.scope_key_count(), 1);

    // A distinct catalog config with the same alias must retain an isolated
    // result cache and therefore owns a second visibility scope key.
    let worker_importer = worker.join("src/entry.ts");
    assert_eq!(
        resolver.resolve("@runtime/value", &worker_importer),
        Some(worker_target)
    );
    assert_eq!(resolver.scope_key_build_count(), 2);
    assert_eq!(resolver.scope_cache_lookup_count(), 2);
    assert_eq!(resolver.scope_key_count(), 2);
}

#[test]
fn automatic_root_catalog_reuses_one_resolver_without_becoming_forced() {
    let root = fixed_root_tsconfig_fixture();
    let config = root.join("tsconfig.json");
    let entry = root.join("src/entry.ts");
    let value = root.join("src/value.ts");
    let package = root.join("packages/without-tsconfig/package.json");
    let external = root.join("src/external.ts");
    let visible = vec![
        config.clone(),
        entry.clone(),
        value.clone(),
        package,
        external.clone(),
    ];
    let catalog = TsConfigCatalog::from_visible(&root, std::slice::from_ref(&root), &visible);

    let provenance = catalog.provenance_for(&entry);
    assert_eq!(
        provenance.config,
        Some(normalize_path(&config.canonicalize().unwrap()))
    );
    assert!(!provenance.forced);
    assert_eq!(catalog.fixed_config(), Some(catalog.config_for(&entry)));
    assert!(catalog.provenance_for(&external).config.is_none());

    let visible = visible.into_iter().collect();
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    assert!(resolver.uses_fixed_resolver());
    assert_eq!(
        resolver.resolve("@root/value", &entry),
        Some(normalize_path(&value.canonicalize().unwrap()))
    );
    assert_eq!(resolver.resolve("@root/value", &external), None);
}

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

    let visible = visible.into_iter().collect();
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
    let resolver = ScopedImportResolver::new(&catalog, &visible.into_iter().collect());
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
    let resolver = ScopedImportResolver::new(&catalog, &visible.into_iter().collect());
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
    let resolver = ScopedImportResolver::new(&catalog, &visible.into_iter().collect());
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
    let resolver = ScopedImportResolver::new(&catalog, &visible.into_iter().collect());
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
    let visible = visible.into_iter().collect();
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
    let resolver = ScopedImportResolver::new(&catalog, &visible.into_iter().collect());
    assert_eq!(resolver.resolve("@hidden/value", &entry), None);
}

#[test]
fn catalog_config_helpers_cover_valid_and_invalid_shapes() {
    let root = workspace_tsconfig_fixture();
    let path = root.join("tsconfig.json");
    assert!(extends_values(&serde_json::json!({}), &path)
        .unwrap()
        .is_empty());
    assert_eq!(
        extends_values(&serde_json::json!({"extends": "./base"}), &path).unwrap(),
        ["./base"]
    );
    assert!(extends_values(&serde_json::json!({"extends": ["./a", 1]}), &path).is_err());
    assert!(extends_values(&serde_json::json!({"extends": true}), &path).is_err());

    assert!(parse_paths(&serde_json::json!([]), &root).is_err());
    assert!(parse_paths(&serde_json::json!({"@x/*": "src/*"}), &root).is_err());
    assert!(parse_paths(&serde_json::json!({"@x/*": [1]}), &root).is_err());
    assert_eq!(
        parse_paths(&serde_json::json!({"@x/*": ["${configDir}/src/*"]}), &root,).unwrap()[0].1[0],
        format!("{}/src/*", root.display())
    );
    assert!(config_relative_path(&serde_json::json!(1), &root, "baseUrl").is_err());
    assert!(
        config_relative_path(&serde_json::json!("src"), &root, "baseUrl")
            .unwrap()
            .ends_with("src")
    );
    assert!(string_list(&serde_json::json!("src"), &path, "include").is_err());
    assert!(string_list(&serde_json::json!([1]), &path, "include").is_err());
    assert_eq!(
        string_list(&serde_json::json!(["src"]), &path, "include").unwrap(),
        ["src"]
    );
    assert!(reference_values(&serde_json::json!({}), &path).is_err());
    assert!(reference_values(&serde_json::json!([{}]), &path).is_err());
    assert!(reference_values(&serde_json::json!([1]), &path).is_err());
    assert_eq!(
        reference_values(&serde_json::json!(["./a", {"path": "./b"}]), &path,).unwrap(),
        ["./a", "./b"]
    );
}

#[test]
fn catalog_extends_merge_preserves_independent_fields() {
    let root = workspace_tsconfig_fixture();
    let mut first = EffectiveConfig::new(root.join("first.json"), root.clone());
    first
        .apply_own(
            &serde_json::json!({
                "compilerOptions": {"paths": {"@x/*": ["src/*"]}},
                "include": ["src/**/*.ts"]
            }),
            &root.join("first.json"),
            &root,
            |value| Ok(root.join(value)),
        )
        .unwrap();
    let mut second = EffectiveConfig::new(root.join("second.json"), root.clone());
    second
        .apply_own(
            &serde_json::json!({"compilerOptions": {"baseUrl": ".", "allowJs": true}}),
            &root.join("second.json"),
            &root,
            |value| Ok(root.join(value)),
        )
        .unwrap();
    let mut child = EffectiveConfig::new(root.join("child.json"), root.clone());
    child.inherit(first);
    child.inherit(second);
    let config = child.tsconfig();
    assert_eq!(config.paths[0].0, "@x/*");
    assert_eq!(config.base_url.as_deref(), Some(root.as_path()));
    assert!(child.matcher().allow_js);
}

#[test]
fn catalog_effective_config_rejects_invalid_compiler_and_project_fields() {
    let root = workspace_tsconfig_fixture();
    let path = root.join("invalid.json");
    for value in [
        serde_json::json!({"compilerOptions": []}),
        serde_json::json!({"compilerOptions": {"allowJs": "yes"}}),
        serde_json::json!({"compilerOptions": {"moduleResolution": 1}}),
        serde_json::json!({"files": "src/a.ts"}),
        serde_json::json!({"include": [1]}),
        serde_json::json!({"exclude": [1]}),
        serde_json::json!({"references": {}}),
    ] {
        let mut config = EffectiveConfig::new(path.clone(), root.clone());
        assert!(config
            .apply_own(&value, &path, &root, |value| Ok(root.join(value)))
            .is_err());
    }
    let mut config = EffectiveConfig::new(path.clone(), root.clone());
    assert!(config
        .apply_own(
            &serde_json::json!({"references": [{"path": "./child"}]}),
            &path,
            &root,
            |_| Err("reference failure".to_string()),
        )
        .is_err());
}

#[test]
fn catalog_matcher_respects_files_includes_excludes_and_out_dir() {
    let root = workspace_tsconfig_fixture();
    let explicit = root.join("explicit.ts");
    let outside_explicit = root.join("../outside-explicit.ts");
    let matcher = ConfigMatcher {
        dir: root.clone(),
        real_dir: root.clone(),
        files: Some(BTreeSet::from([explicit.clone(), outside_explicit.clone()])),
        includes: None,
        excludes: Vec::new(),
        out_dir: None,
        allow_js: false,
    };
    assert!(matcher.owns(&explicit));
    assert!(matcher.owns(&outside_explicit));
    assert!(!matcher.owns(&root.join("other.ts")));
    assert!(!matcher.owns(Path::new("/outside.ts")));

    let matcher = ConfigMatcher {
        dir: root.clone(),
        real_dir: root.clone(),
        files: None,
        includes: Some(vec![GlobRule::new(&root, "src/**/*.ts").unwrap()]),
        excludes: vec![GlobRule::new(&root, "src/excluded/**").unwrap()],
        out_dir: Some(root.join("dist")),
        allow_js: false,
    };
    assert!(matcher.owns(&root.join("src/ok.ts")));
    assert!(!matcher.owns(&root.join("src/excluded/no.ts")));
    assert!(!matcher.owns(&root.join("dist/output.ts")));
    assert!(!matcher.owns(&root.join("src/no.js")));
    assert!(!matcher.owns(&root.join("README.md")));
    let outside_include = GlobRule::new(&root, "../outside/**/*.ts").unwrap();
    assert!(outside_include.matches(&root.join("../outside/src/value.ts")));
    assert!(GlobRule::new(&root, "[").is_none());
    assert!(!GlobRule::new(&root, "")
        .unwrap()
        .matches(Path::new("/outside")));
}

#[test]
fn catalog_matcher_treats_exact_include_files_as_exact() {
    let root = exact_include_files_fixture();
    let config = root.join("tsconfig.json");
    let entry = root.join("src/entry.ts");
    let sibling = root.join("src/sibling.ts");
    let module = root.join("src/module.mts");
    let other_module = root.join("src/other.mts");
    let nested = root.join("directory/nested.ts");
    let visible = vec![
        config.clone(),
        entry.clone(),
        sibling.clone(),
        module.clone(),
        other_module.clone(),
        nested.clone(),
    ];
    let mut builder = CatalogBuilder::new(&root, std::slice::from_ref(&root), &visible, None);
    let matcher = builder.load_effective(&config).unwrap().matcher();

    assert!(matcher.owns(&entry));
    assert!(matcher.owns(&module));
    assert!(matcher.owns(&nested));
    assert!(!matcher.owns(&sibling));
    assert!(!matcher.owns(&other_module));
}

#[test]
fn catalog_matcher_treats_dot_include_as_the_config_directory() {
    let root = include_config_directory_fixture();
    let config = root.join("tsconfig.json");
    let entry = root.join("src/entry.ts");
    let nested = root.join("src/nested/value.ts");
    let script = root.join("scripts/setup.mts");
    let visible = vec![
        config.clone(),
        entry.clone(),
        nested.clone(),
        script.clone(),
    ];
    let mut builder = CatalogBuilder::new(&root, std::slice::from_ref(&root), &visible, None);
    let matcher = builder.load_effective(&config).unwrap().matcher();

    assert!(matcher.owns(&entry));
    assert!(matcher.owns(&nested));
    assert!(matcher.owns(&script));
}

#[test]
fn catalog_matcher_accepts_directory_includes_with_trailing_separators() {
    let root = trailing_directory_include_tsconfig_fixture();
    let config = root.join("tsconfig.json");
    let entry = root.join("src/entry.ts");
    let setup = root.join("scripts/setup.ts");
    let mut builder = CatalogBuilder::new(
        &root,
        std::slice::from_ref(&root),
        &[config.clone(), entry.clone(), setup.clone()],
        None,
    );
    let matcher = builder.load_effective(&config).unwrap().matcher();

    assert!(matcher.owns(&entry));
    assert!(matcher.owns(&setup));
}

#[test]
fn catalog_matcher_matches_absolute_config_dir_include_and_exclude_globs() {
    let root = absolute_glob_rules_fixture();
    let config = root.join("tsconfig.json");
    let included = root.join("src/included.ts");
    let excluded = root.join("src/excluded/value.ts");
    let mut builder = CatalogBuilder::new(
        &root,
        std::slice::from_ref(&root),
        &[config.clone(), included.clone(), excluded.clone()],
        None,
    );
    let matcher = builder.load_effective(&config).unwrap().matcher();

    // `${configDir}` expands before glob matching, so both rules are absolute.
    assert!(matcher.owns(&included));
    assert!(!matcher.owns(&excluded));
    assert!(!matcher.owns(&root.join("outside.ts")));
}

#[test]
fn catalog_builder_helpers_cover_fallback_resolution_shapes() {
    let root = workspace_tsconfig_fixture();
    let builder = CatalogBuilder::new(&root, &[], &[], None);
    assert!(builder.candidates().is_empty());
    assert_eq!(
        builder.resolve_config_value(&root, "./tsconfig").unwrap(),
        root.join("tsconfig.json")
    );
    assert_eq!(
        builder.resolve_config_value(&root, ".").unwrap(),
        root.join("tsconfig.json")
    );
    assert!(builder
        .resolve_package_extends(&root, "@missing/tsconfig")
        .is_err());

    let catalog = TsConfigCatalog::forced(&root, empty_config(&root), None);
    let resolver = ScopedImportResolver::unbounded(&catalog);
    assert!(resolver
        .resolve("./missing", &root.join("src/entry.ts"))
        .is_none());
}

// ── load_tsconfig ─────────────────────────────────────────────────────

#[test]
fn load_tsconfig_parses_paths() {
    let dir = TempDir::new().unwrap();
    let tc = make_tsconfig(dir.path(), r#"{"@utils/*": ["./utils/*"]}"#);
    assert_eq!(tc.paths.len(), 1);
    assert_eq!(tc.paths[0].0, "@utils/*");
}

#[test]
fn load_tsconfig_empty_returns_defaults() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("tsconfig.json");
    write(&p, "{}");
    let tc = load_tsconfig(&p).unwrap();
    assert!(tc.paths.is_empty());
}

#[test]
fn load_tsconfig_invalid_json_errors() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("tsconfig.json");
    write(&p, "{ bad json }");
    assert!(load_tsconfig(&p).is_err());
}

#[test]
fn load_tsconfig_missing_file_errors() {
    let dir = TempDir::new().unwrap();
    assert!(load_tsconfig(&dir.path().join("tsconfig.json")).is_err());
}

// ── find_tsconfig ─────────────────────────────────────────────────────

#[test]
fn find_tsconfig_finds_in_dir() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("tsconfig.json");
    write(&p, "{}");
    assert_eq!(find_tsconfig(dir.path()), Some(p));
}

#[test]
fn find_tsconfig_finds_in_parent() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("tsconfig.json");
    write(&p, "{}");
    let child = dir.path().join("sub").join("dir");
    std::fs::create_dir_all(&child).unwrap();
    assert_eq!(find_tsconfig(&child), Some(p));
}

#[test]
fn find_tsconfig_finds_from_file() {
    let dir = TempDir::new().unwrap();
    let tsc = dir.path().join("tsconfig.json");
    write(&tsc, "{}");
    let file = dir.path().join("src").join("main.mts");
    write(&file, "");
    assert_eq!(find_tsconfig(&file), Some(tsc));
}

// ── resolve_import — relative ─────────────────────────────────────────

#[test]
fn resolves_relative_with_extension() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.mts");
    write(&target, "");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("./utils.mts", &importer, &tc), Some(target));
}

#[test]
fn resolves_relative_no_ext_tries_mts() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.mts");
    write(&target, "");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("./utils", &importer, &tc), Some(target));
}

#[test]
fn resolves_relative_no_ext_falls_back_to_ts() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.ts");
    write(&target, "");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("./utils", &importer, &tc), Some(target));
}

#[test]
fn resolves_relative_dotted_stem_by_appending_known_extension() {
    let root = fixture("dotted-stem");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/button.stories.tsx"));
    let tc = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root,
        base_url: None,
    };
    assert_eq!(
        resolve_import("./button.stories", &importer, &tc),
        Some(target)
    );
}

#[test]
fn resolves_relative_explicit_non_javascript_extension() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let tc = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root,
        base_url: None,
    };
    assert_eq!(resolve_import("./data.json", &importer, &tc), Some(target));
}

#[test]
fn unresolved_explicit_non_javascript_extension_does_not_append_ts_extension() {
    let root = fixture("explicit-css");
    let importer = root.join("src/main.mts");
    let tc = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root,
        base_url: None,
    };
    assert!(resolve_import("./styles.css", &importer, &tc).is_none());
}

#[test]
fn resolves_relative_parent() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("lib.mts");
    write(&target, "");
    // Create the src directory so ../lib.mts resolves through an existing parent.
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    let importer = src.join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("../lib.mts", &importer, &tc), Some(target));
}

#[test]
fn resolves_relative_index_fallback() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils").join("index.mts");
    write(&target, "");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert_eq!(resolve_import("./utils", &importer, &tc), Some(target));
}

#[test]
fn relative_nonexistent_returns_none() {
    let dir = TempDir::new().unwrap();
    let importer = dir.path().join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    assert!(resolve_import("./ghost", &importer, &tc).is_none());
}

#[test]
fn resolution_candidates_cover_absolute_and_queue_compatibility_fallbacks() {
    let root = PathBuf::from("/resolver-candidate-fixture");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let absolute = PathBuf::from("/absolute/missing");
    let standard = ImportResolver::new(&tsconfig)
        .resolution_candidates(absolute.to_str().unwrap(), &root.join("src/main.ts"));
    assert!(standard.contains(&absolute.with_extension("ts")));
    assert!(standard.contains(&absolute.join("index.mts")));

    let queue = ImportResolver::new(&tsconfig).with_queue_compatibility(&root);
    let candidates = queue.resolution_candidates("workers/missing", &root.join("src/main.ts"));
    assert!(candidates.contains(&root.join("workers/missing.ts")));
    assert!(candidates.contains(&root.join("workers/missing/index.mts")));
    let source_candidates =
        queue.resolution_candidates("workers/missing.ts", &root.join("src/main.ts"));
    assert!(source_candidates.contains(&root.join("workers/missing.ts")));
    assert!(source_candidates.contains(&root.join("workers/missing.mts")));
}

// ── resolve_import — aliases ──────────────────────────────────────────

#[test]
fn resolves_alias_exact() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("lib").join("core.mts");
    write(&target, "");
    let tc = make_tsconfig(dir.path(), r#"{"@core": ["./lib/core"]}"#);
    let importer = dir.path().join("main.mts");
    assert_eq!(resolve_import("@core", &importer, &tc), Some(target));
}

#[test]
fn resolves_alias_wildcard() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("utils").join("helpers.mts");
    write(&target, "");
    let tc = make_tsconfig(dir.path(), r#"{"@utils/*": ["./utils/*"]}"#);
    let importer = dir.path().join("main.mts");
    assert_eq!(
        resolve_import("@utils/helpers", &importer, &tc),
        Some(target)
    );
}

#[test]
fn alias_wildcard_with_subpath() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("systems").join("emails").join("queues.mts");
    write(&target, "");
    let tc = make_tsconfig(dir.path(), r#"{"@systems/*": ["./systems/*"]}"#);
    let importer = dir.path().join("main.mts");
    assert_eq!(
        resolve_import("@systems/emails/queues", &importer, &tc),
        Some(target)
    );
}

#[test]
fn alias_nonexistent_returns_none() {
    let dir = TempDir::new().unwrap();
    let tc = make_tsconfig(dir.path(), r#"{"@utils/*": ["./utils/*"]}"#);
    let importer = dir.path().join("main.mts");
    assert!(resolve_import("@utils/ghost", &importer, &tc).is_none());
}

#[test]
fn bare_npm_returns_none() {
    let dir = TempDir::new().unwrap();
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let importer = dir.path().join("main.mts");
    assert!(resolve_import("express", &importer, &tc).is_none());
    assert!(resolve_import("node:path", &importer, &tc).is_none());
}

#[test]
fn catch_all_nonexistent_returns_none() {
    let dir = TempDir::new().unwrap();
    let tc = make_tsconfig(dir.path(), r#"{"*": ["./*"]}"#);
    let importer = dir.path().join("main.mts");
    assert!(resolve_import("some-npm-pkg", &importer, &tc).is_none());
}

#[test]
fn import_resolver_uses_visible_file_set() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.mts");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let resolver = ImportResolver::new(&tc).with_visible(&visible);

    assert_eq!(resolver.resolve("./utils", &importer), Some(target));
}

/// Regression test: `with_visible` (used by every `DepGraph` build and by
/// `server_routes`) must not disable the resolve cache. `resolve()`'s cache-hit
/// branch is a no-op when `cache_enabled` is false, so the "reuses/preserves"
/// tests below pass on identical *results* even with caching off — they don't
/// prove memoization happened. This asserts the cache is actually populated.
#[test]
fn import_resolver_with_visible_keeps_cache_enabled() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("src").join("utils.mts");
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let visible: HashSet<PathBuf> = [target].into();
    let resolver = ImportResolver::new(&tc).with_visible(&visible);
    assert!(resolver.cache_enabled);

    resolver.resolve("./utils", &importer);

    assert_eq!(resolver.cache.len(), 1);
}

/// Regression test: calling `with_visible` on a resolver that already has
/// cached entries (resolved against the real filesystem, or an earlier
/// `visible` set) must not leak those stale answers into the new visibility
/// scope — `with_visible` consumes and returns `Self`, so a reused resolver's
/// `cache` would otherwise carry answers computed under different visibility.
#[test]
fn import_resolver_with_visible_clears_stale_cache_entries() {
    let dir = TempDir::new().unwrap();
    let target = normalize_path(&dir.path().join("src").join("utils.mts"));
    let importer = dir.path().join("src").join("main.mts");
    write(&target, "");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tc);
    assert_eq!(resolver.resolve("./utils", &importer), Some(target));

    let visible: HashSet<PathBuf> = HashSet::new();
    let resolver = resolver.with_visible(&visible);

    assert!(resolver.resolve("./utils", &importer).is_none());
}

#[test]
fn import_resolver_cache_reuses_present_result() {
    let dir = TempDir::new().unwrap();
    let target = normalize_path(&dir.path().join("src").join("utils.mts"));
    let importer = dir.path().join("src").join("main.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let resolver = ImportResolver::new(&tc).with_visible(&visible);

    assert_eq!(resolver.resolve("./utils", &importer), Some(target.clone()));
    assert_eq!(resolver.resolve("./utils", &importer), Some(target));
    assert!(resolver.resolve("./utils.mts", &importer).is_some());
    assert!(resolver.resolve("./missing.mts", &importer).is_none());
}

#[test]
fn import_resolver_cache_preserves_missing_result() {
    let dir = TempDir::new().unwrap();
    let importer = dir.path().join("src").join("main.mts");
    let target = dir.path().join("src").join("utils.mts");
    let tc = TsConfig {
        dir: dir.path().to_path_buf(),
        paths: vec![],
        paths_dir: dir.path().to_path_buf(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tc);

    assert!(resolver.resolve("./utils", &importer).is_none());
    write(&target, "");
    assert!(resolver.resolve("./utils", &importer).is_none());
}

#[test]
fn import_resolver_reports_exact_cached_work_for_hits_and_misses() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let resolver =
        ImportResolver::new_observed(&config, Some(observer.clone())).with_visible(&visible);

    assert_eq!(
        resolver.resolve("./data.json", &importer),
        Some(target.clone())
    );
    assert_eq!(resolver.resolve("./data.json", &importer), Some(target));
    assert!(resolver.resolve("./missing.json", &importer).is_none());
    assert!(resolver.resolve("./missing.json", &importer).is_none());

    let work = observer.snapshot().work;
    assert_eq!(work["resolver.requests"], 4);
    assert_eq!(work["resolver.computations"], 2);
    assert_eq!(work["resolver.cache_hits"], 2);
    assert_eq!(work["resolver.resolved"], 1);
    assert_eq!(work["resolver.unresolved"], 1);
}

#[test]
fn import_resolver_single_flights_concurrent_hits_and_misses() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let resolver =
        ImportResolver::new_observed(&config, Some(observer.clone())).with_visible(&visible);
    let requests_per_key = 32;
    let barrier = std::sync::Barrier::new(requests_per_key * 2);

    std::thread::scope(|scope| {
        for request in 0..requests_per_key * 2 {
            let resolver = &resolver;
            let importer = &importer;
            let target = &target;
            let barrier = &barrier;
            scope.spawn(move || {
                barrier.wait();
                if request % 2 == 0 {
                    assert_eq!(
                        resolver.resolve("./data.json", importer),
                        Some(target.clone())
                    );
                } else {
                    assert!(resolver.resolve("./missing.json", importer).is_none());
                }
            });
        }
    });

    let work = observer.snapshot().work;
    assert_eq!(work["resolver.requests"], 64);
    assert_eq!(work["resolver.computations"], 2);
    assert_eq!(work["resolver.cache_hits"], 62);
    assert_eq!(work["resolver.resolved"], 1);
    assert_eq!(work["resolver.unresolved"], 1);
}

#[test]
fn import_resolver_session_reuses_only_exact_resolution_scopes() {
    let root = fixture("explicit-json");
    let importer = root.join("src/main.mts");
    let target = normalize_path(&root.join("src/data.json"));
    let visible: HashSet<PathBuf> = [target.clone()].into();
    let hidden = HashSet::new();
    let config = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root,
        base_url: None,
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));

    let first = ImportResolver::new_in_session(&config, Some(&visible), &session);
    assert_eq!(
        first.resolve("./data.json", &importer),
        Some(target.clone())
    );
    assert!(first.resolve("./missing.json", &importer).is_none());
    let second = ImportResolver::new_in_session(&config, Some(&visible), &session);
    assert_eq!(
        second.resolve("./data.json", &importer),
        Some(target.clone())
    );
    assert!(second.resolve("./missing.json", &importer).is_none());

    let hidden_scope = ImportResolver::new_in_session(&config, Some(&hidden), &session);
    assert!(hidden_scope.resolve("./data.json", &importer).is_none());
    let filesystem_scope = ImportResolver::new_in_session(&config, None, &session);
    assert_eq!(
        filesystem_scope.resolve("./data.json", &importer),
        Some(target)
    );

    let work = observer.snapshot().work;
    assert_eq!(work["resolver.requests"], 6);
    assert_eq!(work["resolver.computations"], 4);
    assert_eq!(work["resolver.unique_keys"], 4);
    assert_eq!(work["resolver.cache_hits"], 2);
    assert_eq!(work["resolver.resolved"], 2);
    assert_eq!(work["resolver.unresolved"], 2);
}

// ── match_alias ───────────────────────────────────────────────────────

#[test]
fn match_alias_exact() {
    assert_eq!(match_alias("@core", "@core"), Some(String::new()));
    assert_eq!(match_alias("@core", "@other"), None);
}

#[test]
fn match_alias_wildcard() {
    assert_eq!(match_alias("@u/*", "@u/foo"), Some("foo".to_string()));
    assert_eq!(match_alias("@u/*", "@v/foo"), None);
}

#[test]
fn match_alias_wildcard_subpath() {
    assert_eq!(
        match_alias("@sys/*", "@sys/emails/queues"),
        Some("emails/queues".to_string())
    );
}

mod extends;
