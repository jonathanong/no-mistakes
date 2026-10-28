//! Low annotation imports reuse the scoped request resolver and prepared file map.
use super::CheckFileFacts;
use crate::codebase::ts_resolver::ScopedImportResolver;
use crate::codebase::ts_source::FileIdMap;
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::fx::PathSet;

pub(super) fn has_candidates(files: &FileIdMap<CheckFileFacts>) -> bool {
    files.values().any(|file| {
        file.query_annotation_heads
            .as_ref()
            .is_some_and(|heads| heads.has_pending_imports())
    })
}

pub(super) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolver: &ScopedImportResolver<'_>,
    workspace: &IndexedWorkspaceMap,
    visible: &PathSet,
) {
    crate::codebase::postgres::query_annotation::heads::project(files, |specifier, from| {
        if let Some(target) = resolver.resolve(specifier, from) {
            return visible.contains(&target).then_some(target);
        }
        workspace.resolve_specifier_from_file_visible(specifier, from, visible)
    });
}
