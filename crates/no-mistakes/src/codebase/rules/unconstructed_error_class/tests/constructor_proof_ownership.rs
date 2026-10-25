use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_resolver::declarations::DeclarationResolver;
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};

#[test]
fn declaration_projection_memoizes_positive_and_negative_results_in_the_session() {
    let root = fixture()
        .parent()
        .unwrap()
        .join("shared-package-targets/fixture");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(observer.clone()));
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let resolver = ImportResolver::new_in_session(&config, None, &session);
    let declarations =
        DeclarationResolver::new(&session, &root, &resolver, &config, None, &sources, None);
    let importer = root.join("src/packages.ts");
    for specifier in ["pg", "unavailable-package"] {
        let first = declarations.resolve(specifier, &importer);
        assert_eq!(first.is_some(), specifier == "pg");
        let reads = sources.physical_read_count();
        let parses = sources.json_parse_count();
        assert_eq!(declarations.resolve(specifier, &importer), first);
        assert_eq!(sources.physical_read_count(), reads);
        assert_eq!(sources.json_parse_count(), parses);
    }
    assert_eq!(
        observer.snapshot().work["resolver.declaration_resolutions"],
        2
    );
}

#[test]
fn workspace_catalog_identity_and_visibility_bound_declaration_resolution() {
    use crate::codebase::dependencies::graph::GraphFiles;
    use crate::codebase::ts_resolver::WorkspaceFallbackResolver;
    use crate::codebase::workspaces::IndexedWorkspaceMap;
    let root = fixture()
        .parent()
        .unwrap()
        .join("workspace-declaration-projection/fixture");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(observer.clone()));
    let visible = session.visible_paths(&root);
    let files = GraphFiles::from_files(visible.paths_for(&root).as_ref().clone());
    let sources = visible.source_store_for(&root);
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let importer = root.join("app/main.ts");
    let workspace = session.workspace(&root);
    let base = ImportResolver::new_in_session(&config, Some(&files), &session);
    let scoped = WorkspaceFallbackResolver::new(&base, &workspace, &files);
    let declarations = DeclarationResolver::new(
        &session,
        &root,
        &scoped,
        &config,
        None,
        &sources,
        Some(&workspace),
    );
    let target = Some(root.join("packages/declarations/index.d.ts"));
    assert_eq!(
        declarations.resolve("@fixture/declarations", &importer),
        target
    );
    let cloned_workspace = workspace.as_ref().clone();
    let cloned = WorkspaceFallbackResolver::new(&base, &cloned_workspace, &files);
    let reused = DeclarationResolver::new(
        &session,
        &root,
        &cloned,
        &config,
        None,
        &sources,
        Some(&cloned_workspace),
    );
    assert_eq!(reused.resolve("@fixture/declarations", &importer), target);
    assert_eq!(
        observer.snapshot().work["resolver.declaration_resolutions"],
        1
    );

    // A different prepared workspace under identical roots/config/visibility
    // must not receive the first projection's cached workspace path.
    let empty = IndexedWorkspaceMap::default();
    let external = WorkspaceFallbackResolver::new(&base, &empty, &files);
    let installed = DeclarationResolver::new(
        &session,
        &root,
        &external,
        &config,
        None,
        &sources,
        Some(&empty),
    );
    assert_eq!(
        installed.resolve("@fixture/declarations", &importer),
        Some(root.join("node_modules/@fixture/declarations/index.d.ts"))
    );

    // A known workspace package outside this graph cannot be replaced by an
    // installed namesake merely because that namesake has declarations.
    let hidden = GraphFiles::from_files(vec![importer.clone()]);
    // Keep the inner resolver broad: only workspace fallback visibility narrows.
    let hidden_scoped = WorkspaceFallbackResolver::new(&base, &workspace, &hidden);
    let hidden_declarations = DeclarationResolver::new(
        &session,
        &root,
        &hidden_scoped,
        &config,
        None,
        &sources,
        Some(&workspace),
    );
    assert_eq!(
        hidden_declarations.resolve("@fixture/declarations", &importer),
        None
    );
    assert_eq!(
        observer.snapshot().work["resolver.declaration_resolutions"],
        3
    );
}
