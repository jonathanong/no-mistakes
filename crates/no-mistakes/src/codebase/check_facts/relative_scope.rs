use super::CheckFileFacts;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::postgres::{
    project_relative_scoped_facts, query_annotation::project as annotation_project,
};
use crate::codebase::ts_resolver::{ImportResolver, ScopedImportResolver, TsConfigCatalog};
use crate::codebase::ts_source::{FileIdMap, SourceStore};
use crate::fx::PathSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod head_helpers;
mod imported_sql;
mod setup;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
use setup::remember_package_root;
use setup::{cached_package_root, resolve_exact, tsconfig_for, workspace_for};
mod unproven;
pub(super) use unproven::count_unproven_member_queries;
#[doc(hidden)]
pub use unproven::unproven_member_query_warning;

/// Promote relative scoped-executor candidates using the request resolver cache.
/// Files with no candidates are left untouched. A missing package root drops them.
pub(super) fn project_relative_executor_scopes(
    session: &AnalysisSession,
    root: &Path,
    sources: &SourceStore,
    catalog: Option<&TsConfigCatalog>,
    files: &mut FileIdMap<CheckFileFacts>,
) -> u32 {
    project_with_annotation_overlay(session, root, sources, catalog, files, None)
}

pub(super) fn project_with_annotation_overlay(
    session: &AnalysisSession,
    root: &Path,
    sources: &SourceStore,
    catalog: Option<&TsConfigCatalog>,
    files: &mut FileIdMap<CheckFileFacts>,
    overlay: Option<&mut annotation_project::SyntaxOverlay<'_>>,
) -> u32 {
    let low_helpers = files
        .into_iter()
        .any(|(_, file)| !file.sql_identifier_syntax.recovery.calls.is_empty());
    let annotation = files
        .into_iter()
        .any(|(_, file)| !file.query_annotation.is_empty());
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
    let fallback_catalog = std::cell::OnceCell::new();
    let scoped = std::cell::OnceCell::new();
    let scoped_resolver = || {
        scoped.get_or_init(|| {
            let catalog = catalog.unwrap_or_else(|| {
                fallback_catalog.get_or_init(|| {
                    TsConfigCatalog::from_visible_and_sources_with_workspace(
                        root,
                        &[root.to_path_buf()],
                        sources.inventory().paths().as_ref(),
                        sources,
                        &workspace,
                    )
                })
            });
            ScopedImportResolver::new_in_session(catalog, &visible, session)
        })
    };
    let annotation_catalog = overlay.filter(|_| annotation).map(|overlay| {
        let resolver = scoped_resolver();
        // A lowering panic invalidates the file before other rules can project its facts.
        annotation_project::prepare_syntax(
            files,
            |specifier, from| resolve_exact(resolver, &workspace, &visible, specifier, from),
            overlay,
        )
    });
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
        let resolver = scoped_resolver();
        if head_helpers {
            head_helpers::project(files, resolver, &workspace, &visible);
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
                |specifier, from| resolve_exact(resolver, &workspace, &visible, specifier, from),
            );
        }
        if annotation {
            crate::perf_trace::trace("postgres.annotation.project", || {
                annotation_project::project_with_catalog(
                    files,
                    |specifier, from| {
                        resolve_exact(resolver, &workspace, &visible, specifier, from)
                    },
                    annotation_catalog,
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

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
