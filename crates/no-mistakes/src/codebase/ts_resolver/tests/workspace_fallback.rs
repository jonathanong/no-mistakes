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
fn workspace_specifiers_resolve_after_the_wrapped_resolver_gives_up() {
    let importer = PathBuf::from("/repo/apps/web/src/entry.ts");
    let sibling = PathBuf::from("/repo/apps/web/src/local.ts");
    let entry = PathBuf::from("/repo/packages/core/src/index.ts");
    let visible: crate::fx::PathSet = [importer.clone(), sibling.clone(), entry.clone()]
        .into_iter()
        .collect();
    let tsconfig = tsconfig();
    let inner = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = workspace(&entry);
    let resolver = WorkspaceFallbackResolver::new(&inner, &workspace);

    assert_eq!(inner.resolve("@fixture/core", &importer), None);
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "@fixture/core", &importer),
        Some(entry)
    );
    // The wrapped resolver still answers first for everything it can resolve.
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "./local", &importer),
        Some(sibling)
    );
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "@fixture/missing", &importer),
        None
    );
}

#[test]
fn everything_but_resolution_is_delegated_to_the_wrapped_resolver() {
    let importer = PathBuf::from("/repo/apps/web/src/entry.ts");
    let sibling = PathBuf::from("/repo/apps/web/src/local.ts");
    let entry = PathBuf::from("/repo/packages/core/src/index.ts");
    let visible: crate::fx::PathSet = [importer.clone(), sibling.clone(), entry.clone()]
        .into_iter()
        .collect();
    let tsconfig = tsconfig();
    let inner = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = workspace(&entry);
    let resolver = WorkspaceFallbackResolver::new(&inner, &workspace);

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
fn without_a_visible_file_universe_workspace_specifiers_stay_unresolved() {
    let importer = PathBuf::from("/repo/apps/web/src/entry.ts");
    let entry = PathBuf::from("/repo/packages/core/src/index.ts");
    let tsconfig = tsconfig();
    let inner = ImportResolver::new(&tsconfig);
    let workspace = workspace(&entry);
    let resolver = WorkspaceFallbackResolver::new(&inner, &workspace);

    assert!(ImportResolverFacade::visible_files(&resolver).is_none());
    assert_eq!(
        ImportResolverFacade::resolve(&resolver, "@fixture/core", &importer),
        None
    );
}
