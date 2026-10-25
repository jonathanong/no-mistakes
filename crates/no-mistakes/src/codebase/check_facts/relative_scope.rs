use super::CheckFileFacts;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::postgres::{
    package_name, package_root_for_specifier, project_relative_scoped_facts,
};
use crate::codebase::ts_resolver::{ImportResolver, ScopedImportResolver, TsConfigCatalog};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
use crate::codebase::workspaces::IndexedWorkspaceMap;
use crate::fx::PathSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod imported_sql;
mod setup;
use setup::{tsconfig_for, workspace_for};
mod unproven;
pub(super) use unproven::count_unproven_member_queries;
#[doc(hidden)]
pub use unproven::unproven_member_query_warning;

/// Promote relative scoped-executor candidates using the request resolver cache.
///
/// Files with no candidates are left untouched. A missing package root drops them.
pub(super) fn project_relative_executor_scopes(
    session: &AnalysisSession,
    root: &Path,
    sources: &SourceStore,
    catalog: Option<&TsConfigCatalog>,
    files: &mut FileIdMap<CheckFileFacts>,
) -> u32 {
    let low_helpers = files
        .into_iter()
        .any(|(_, file)| !file.sql_identifier_syntax.recovery.calls.is_empty());
    let annotation = files
        .into_iter()
        .any(|(_, file)| !file.query_annotation.is_empty());
    if !low_helpers
        && !has_relative_candidates(files)
        && !annotation
        && !imported_sql::has_candidates(files)
    {
        return finish_unproven(files);
    }
    let tsconfig = tsconfig_for(session, root, sources);
    let workspace = workspace_for(session, root, sources);
    let mut visible = PathSet::default();
    for path in sources.inventory().paths().iter() {
        visible.insert(path.clone());
    }
    let resolver = ImportResolver::new_in_session(tsconfig.as_ref(), Some(&visible), session);
    let mut package_roots = HashMap::<String, PathBuf>::new();
    for (path, facts) in &mut *files {
        for (options, embedded) in &mut facts.embedded_sql {
            if embedded.pending_relative.candidates.is_empty() {
                continue;
            }
            let package_root = cached_package_root(
                &mut package_roots,
                &options.import_specifier,
                path,
                &workspace,
                &resolver,
            );
            let source = sources
                .read_path(path)
                .unwrap_or_else(|_| std::sync::Arc::<str>::from(""));
            project_relative_scoped_facts(
                embedded,
                source.as_ref(),
                package_root.as_deref(),
                |specifier| resolver.resolve(specifier, path),
            );
        }
    }
    let imported = imported_sql::has_candidates(files);
    if imported || annotation || low_helpers {
        // Compatibility collectors lack a boundary catalog, but still reuse
        // their supplied inventory, sources and workspace. Prepared entrypoints
        // always pass their existing catalog, preserving forced-config semantics.
        // Imported helpers resolve through the same per-importing-file
        // selection, so a package's own `paths` win over the root's.
        let fallback;
        let catalog = match catalog {
            Some(catalog) => catalog,
            None => {
                fallback = TsConfigCatalog::from_visible_and_sources_with_workspace(
                    root,
                    &[root.to_path_buf()],
                    sources.inventory().paths().as_ref(),
                    sources,
                    &workspace,
                );
                &fallback
            }
        };
        let resolver = ScopedImportResolver::new_in_session(catalog, &visible, session);
        if imported {
            // A pnpm workspace package name resolves through the workspace
            // map, as it does for every other import-following analysis.
            imported_sql::project(files, |specifier, from| {
                resolver.resolve(specifier, from).or_else(|| {
                    workspace.resolve_specifier_from_file_visible(specifier, from, &visible)
                })
            });
        }
        if low_helpers {
            crate::codebase::postgres::sql_identifier_syntax::helper_recovery::project(
                files,
                |specifier, from| {
                    if let Some(target) = resolver.resolve(specifier, from) {
                        return visible.contains(&target).then_some(target);
                    }
                    workspace.resolve_specifier_from_file_visible(specifier, from, &visible)
                },
            );
        }
        if annotation {
            crate::perf_trace::trace("postgres.annotation.project", || {
                crate::codebase::postgres::query_annotation::project::project(
                    files,
                    |specifier, from| {
                        // Canonical visibility also admits symlink aliases that do
                        // not identify a prepared source. Keep those unresolved:
                        // only an exact inventory identity proves an external edge.
                        // Do not replace that hit with a workspace target.
                        // Package names the scoped resolver misses use the session
                        // workspace map, as imported SQL helpers do. No target
                        // from either source stays unresolved.
                        if let Some(target) = resolver.resolve(specifier, from) {
                            return visible.contains(&target).then_some(target);
                        }
                        workspace.resolve_specifier_from_file_visible(specifier, from, &visible)
                    },
                );
            });
        }
    }
    finish_unproven(files)
}

fn finish_unproven(files: &FileIdMap<CheckFileFacts>) -> u32 {
    let count = unproven::count_unproven_member_queries(files.values());
    unproven::note_unproven_member_queries(count);
    count
}

fn has_relative_candidates(files: &FileIdMap<CheckFileFacts>) -> bool {
    files.into_iter().any(|(_, facts)| {
        facts
            .embedded_sql
            .iter()
            .any(|(_, embedded)| !embedded.pending_relative.candidates.is_empty())
    })
}

fn cached_package_root(
    cache: &mut HashMap<String, PathBuf>,
    specifier: &str,
    file: &Path,
    workspace: &IndexedWorkspaceMap,
    resolver: &ImportResolver<'_>,
) -> Option<PathBuf> {
    if let Some(cached) = cache.get(specifier) {
        return Some(cached.clone());
    }
    let resolved = package_root_for_specifier(specifier, file, workspace, |spec, from| {
        resolver.resolve(spec, from)
    });
    remember_package_root(cache, specifier, workspace, resolved)
}

/// Cache a root only when `package_by_name` already named it. A fallback that
/// walked up from one file is not stable for the next importer.
pub(super) fn remember_package_root(
    cache: &mut HashMap<String, PathBuf>,
    specifier: &str,
    workspace: &IndexedWorkspaceMap,
    resolved: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(root) = resolved.clone() {
        if package_name(specifier).is_some_and(|name| workspace.package_by_name(name).is_some()) {
            cache.insert(specifier.to_string(), root);
        }
    }
    resolved
}

#[cfg(test)]
mod tests;
