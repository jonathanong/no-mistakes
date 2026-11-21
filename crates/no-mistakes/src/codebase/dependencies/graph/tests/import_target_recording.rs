use super::super::{import_target_for, GraphFiles, ImportTarget};
use crate::codebase::ts_resolver::{ImportResolver, TsConfig};
use std::path::{Path, PathBuf};

fn path(text: &str) -> PathBuf {
    PathBuf::from(text)
}

fn tsconfig(paths: Vec<(String, Vec<String>)>) -> TsConfig {
    let root = path("/repo");
    TsConfig {
        dir: root.clone(),
        paths,
        paths_dir: root,
        base_url: None,
    }
}

fn workspace(entry: &Path) -> crate::codebase::workspaces::IndexedWorkspaceMap {
    crate::codebase::workspaces::IndexedWorkspaceMap::from_packages(vec![
        crate::codebase::workspaces::WorkspacePackage {
            name: "@widgets/orders".to_string(),
            dir: path("/repo/orders"),
            entry: Some(entry.to_path_buf()),
            exports: None,
            imports: None,
        },
    ])
}

fn recorded(
    specifier: &str,
    importer: &Path,
    resolver: &ImportResolver<'_>,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    files: &GraphFiles,
) -> ImportTarget {
    let classification = resolver.classify_import(specifier, importer, workspace, files);
    import_target_for(
        None,
        &classification,
        resolver,
        importer,
        specifier,
        workspace,
        files,
    )
}

#[test]
fn workspace_files_are_recorded_and_packages_are_external() {
    let importer = path("/repo/widgets/list.ts");
    let entry = path("/repo/orders/index.ts");
    let files = GraphFiles::from_files(vec![importer.clone(), entry.clone()]);
    let packages = workspace(&entry);
    let config = tsconfig(Vec::new());
    let resolver = ImportResolver::new(&config);
    assert_eq!(
        recorded("@widgets/orders", &importer, &resolver, &packages, &files),
        ImportTarget::File(entry)
    );
    assert_eq!(
        recorded("node:fs", &importer, &resolver, &packages, &files),
        ImportTarget::External
    );
    assert_eq!(
        recorded("orders-widget", &importer, &resolver, &packages, &files),
        ImportTarget::External
    );
    assert_eq!(
        recorded("./missing-widget", &importer, &resolver, &packages, &files),
        ImportTarget::Unresolved
    );

    let alias_config = tsconfig(vec![(
        "@widgets/*".to_string(),
        vec!["./widgets/*".to_string()],
    )]);
    let aliased = ImportResolver::new(&alias_config);
    assert_eq!(
        recorded(
            "@widgets/missing-item",
            &importer,
            &aliased,
            &packages,
            &files
        ),
        ImportTarget::Unresolved
    );
}
