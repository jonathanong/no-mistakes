use super::{ImportRow, Status};
use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::ts_resolver::ImportResolver;
use std::path::Path;

pub(super) fn kind_str(kind: ImportKind) -> &'static str {
    match kind {
        ImportKind::Static => "static",
        ImportKind::Type => "type",
        ImportKind::Dynamic => "dynamic",
        ImportKind::Require => "require",
        ImportKind::RequireResolve => "require-resolve",
    }
}

/// Declaration files only satisfy type-only references because they do not
/// emit a runtime module.
fn is_declaration_file(path: &Path) -> bool {
    let name = path.to_string_lossy();
    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
}

pub(super) fn classify(
    imp: &ExtractedImport,
    target: &super::super::shared::Target,
    resolver: &ImportResolver,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> ImportRow {
    classify_for_importer(
        imp,
        &target.abs_file,
        &target.root,
        resolver,
        workspace,
        target.visible_files(),
    )
}

fn classify_for_importer(
    imp: &ExtractedImport,
    importing_file: &Path,
    root: &Path,
    resolver: &ImportResolver,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
) -> ImportRow {
    if imp.computed {
        return ImportRow {
            specifier: imp.specifier.clone(),
            kind: kind_str(imp.kind),
            status: Status::Unresolved,
            resolved: None,
            computed: true,
        };
    }
    let classification =
        resolver.classify_import(&imp.specifier, importing_file, workspace, visible);
    // A configured alias remains authoritative even when its spelling overlaps
    // a workspace package. Workspace targets use the graph's visible projection.
    let resolved = classification
        .resolver_path()
        .or_else(|| {
            (!resolver.matches_alias(&imp.specifier))
                .then(|| classification.workspace_path())
                .flatten()
        })
        .filter(|path| imp.kind == ImportKind::Type || !is_declaration_file(path));
    let status = if resolved.is_some() {
        Status::Resolved
    } else if imp.specifier.starts_with('.')
        || resolver.matches_alias(&imp.specifier)
        || workspace.recognizes_specifier_from(&imp.specifier, importing_file)
    {
        Status::Unresolved
    } else {
        Status::External
    };
    ImportRow {
        specifier: imp.specifier.clone(),
        kind: kind_str(imp.kind),
        status,
        resolved: resolved.map(|path| super::super::shared::rel_str(path, root)),
        computed: false,
    }
}

/// Classify imports from facts already collected by a prepared project
/// analysis. The resolver matches the standalone resolve-check resolver for
/// this importer, preserving local/alias precedence and workspace visibility.
pub(super) fn classify_prepared(
    imp: &ExtractedImport,
    importing_file: &Path,
    root: &Path,
    resolver: &ImportResolver,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
    visible: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
) -> ImportRow {
    classify_for_importer(imp, importing_file, root, resolver, workspace, visible)
}
