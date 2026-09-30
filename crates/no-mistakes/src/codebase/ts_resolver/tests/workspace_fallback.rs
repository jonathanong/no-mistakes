use super::*;
use crate::codebase::workspaces::{IndexedWorkspaceMap, WorkspacePackage};

fn tsconfig() -> TsConfig {
    TsConfig {
        dir: PathBuf::from("/repo/apps/web"),
        paths: vec![],
        paths_dir: PathBuf::from("/repo/apps/web"),
        base_url: None,
    }
}

fn workspace(entry: &Path) -> IndexedWorkspaceMap {
    IndexedWorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@fixture/core".to_string(),
        dir: PathBuf::from("/repo/packages/core"),
        entry: Some(entry.to_path_buf()),
        exports: None,
        imports: None,
    }])
}

#[test]
fn workspace_specifiers_resolve_last_and_the_rest_is_delegated() {
    let importer = PathBuf::from("/repo/apps/web/src/entry.ts");
    let sibling = PathBuf::from("/repo/apps/web/src/local.ts");
    let entry = PathBuf::from("/repo/packages/core/src/index.ts");
    let visible: crate::fx::PathSet = [importer.clone(), sibling.clone(), entry.clone()]
        .into_iter()
        .collect();
    let tsconfig = tsconfig();
    let inner = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = workspace(&entry);
    let resolver = WorkspaceFallbackResolver::new(&inner, &workspace, &visible);

    assert_eq!(inner.resolve("@fixture/core", &importer), None);
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "@fixture/core", &importer),
        Some(entry)
    );
    // The wrapped resolver still answers first for everything it can resolve.
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "./local", &importer),
        Some(sibling.clone())
    );
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "@fixture/missing", &importer),
        None
    );

    // Everything but resolution is the wrapped resolver's answer.
    assert!(ImportResolverFacade::visible_files(&resolver).is_some());
    assert_eq!(
        ImportResolverFacade::resolution_candidates(&resolver, "./local", &importer),
        inner.resolution_candidates("./local", &importer)
    );
    let classified = ImportResolverFacade::classify_import(
        &resolver, "./local", &importer, &workspace, &visible,
    );
    assert_eq!(classified.resolver_path(), Some(sibling.as_path()));
}

#[test]
fn the_supplied_lookup_not_the_wrapped_resolver_decides_workspace_visibility() {
    let importer = PathBuf::from("/repo/apps/web/src/entry.ts");
    let entry = PathBuf::from("/repo/packages/core/src/index.ts");
    let tsconfig = tsconfig();
    let inner = ImportResolver::new(&tsconfig);
    let workspace = workspace(&entry);
    let graph: crate::fx::PathSet = [importer.clone(), entry.clone()].into_iter().collect();
    let without_entry: crate::fx::PathSet = [importer.clone()].into_iter().collect();

    let resolver = WorkspaceFallbackResolver::new(&inner, &workspace, &graph);
    assert!(ImportResolverFacade::visible_files(&resolver).is_none());
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "@fixture/core", &importer),
        Some(entry)
    );

    let resolver = WorkspaceFallbackResolver::new(&inner, &workspace, &without_entry);
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "@fixture/core", &importer),
        None
    );
}

#[test]
fn package_imports_resolve_from_files_inside_the_owning_package() {
    let importer = PathBuf::from("/repo/packages/core/src/index.ts");
    let internal = PathBuf::from("/repo/packages/core/src/internal.ts");
    let outside = PathBuf::from("/repo/apps/web/src/entry.ts");
    let visible: crate::fx::PathSet = [importer.clone(), internal.clone(), outside.clone()]
        .into_iter()
        .collect();
    let tsconfig = tsconfig();
    let inner = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = IndexedWorkspaceMap::from_packages(vec![WorkspacePackage {
        name: "@fixture/core".to_string(),
        dir: PathBuf::from("/repo/packages/core"),
        entry: Some(importer.clone()),
        exports: None,
        imports: Some(serde_json::json!({ "#internal": "./src/internal.ts" })),
    }]);
    let resolver = WorkspaceFallbackResolver::new(&inner, &workspace, &visible);

    assert_eq!(inner.resolve("#internal", &importer), None);
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "#internal", &importer),
        Some(internal)
    );
    // `#name` is scoped to the package that owns the importing file.
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "#internal", &outside),
        None
    );
}
