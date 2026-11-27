use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{workspace_setup as setup, CheckFileFacts};
use crate::codebase::postgres::project_relative_scoped_facts;
use crate::codebase::ts_resolver::{ImportResolver, ScopedImportResolver, TsConfigCatalog};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
use crate::fx::PathSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod head_helpers;
mod imported_sql;
mod package_roots;
use package_roots::cached_package_root;
use setup::{tsconfig_for, workspace_for};
mod unproven;
pub(crate) fn count_unproven_in_view(files: &FileIdMap<std::sync::Arc<CheckFileFacts>>) -> u32 {
    unproven::count_unproven_member_queries(files.values().map(std::sync::Arc::as_ref))
}

/// Promote relative scoped-executor candidates using the request resolver cache.
///
/// Files with no candidates are left untouched. A missing package root drops them.
pub(crate) fn project_relative_executor_scopes(
    session: &AnalysisSession,
    root: &Path,
    sources: &SourceStore,
    catalog: Option<&TsConfigCatalog>,
    files: &mut FileIdMap<CheckFileFacts>,
) -> u32 {
    let low_helpers = files.into_iter().any(|(_, file)| {
        !file
            .postgres()
            .sql_identifier_syntax
            .recovery
            .calls
            .is_empty()
    });
    let annotation = files
        .into_iter()
        .any(|(_, file)| !file.postgres().query_annotation.is_empty());
    let head_helpers = head_helpers::has_candidates(files);
    if !low_helpers
        && !has_relative_candidates(files)
        && !annotation
        && !head_helpers
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
        let Some(postgres) = facts.postgres_if_present_mut() else {
            continue;
        };
        for (options, embedded) in &mut postgres.embedded_sql {
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
                .read_source_path(path)
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
    if imported || annotation || low_helpers || head_helpers {
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
        if head_helpers {
            head_helpers::project(files, &resolver, &workspace, &visible);
        }
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
            .postgres()
            .embedded_sql
            .iter()
            .any(|(_, embedded)| !embedded.pending_relative.candidates.is_empty())
    })
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
