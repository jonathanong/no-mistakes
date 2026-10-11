use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::rules::test_no_unmocked_dynamic_imports::resolve_mock_specifiers;
use crate::codebase::ts_resolver::ScopedImportResolver;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(in super::super) struct Request<'a> {
    pub(in super::super) test_file: &'a Path,
    pub(in super::super) graph: &'a DepGraph,
    pub(in super::super) graph_files: &'a GraphFiles,
    pub(in super::super) visible_files: &'a crate::fx::PathSet,
    pub(in super::super) resolver: &'a ScopedImportResolver<'a>,
    pub(in super::super) shared: &'a CheckFactMap,
    pub(in super::super) excluded: &'a HashSet<PathBuf>,
}

pub(in super::super) fn collect(request: Request<'_>) -> anyhow::Result<HashSet<PathBuf>> {
    super::super::super::setup_helper_graph::collect(
        request.graph,
        request.test_file,
        request.visible_files,
        request.excluded,
        |file| {
            Ok(request
                .shared
                .ts
                .get(file)
                .and_then(|facts| facts.dynamic_imports.as_ref())
                .map(|dynamic| {
                    resolve_mock_specifiers(
                        &dynamic.mock_specifiers,
                        file,
                        request.resolver,
                        Some(request.graph_files),
                    )
                })
                .unwrap_or_default())
        },
    )
}

pub(in super::super) fn collect_strict(request: Request<'_>) -> anyhow::Result<HashSet<PathBuf>> {
    super::super::super::setup_helper_graph::collect(
        request.graph,
        request.test_file,
        request.visible_files,
        request.excluded,
        |file| {
            let Some(facts) = request.shared.ts.get(file) else {
                anyhow::bail!(
                    "missing shared facts for imported setup helper {}",
                    file.display()
                );
            };
            if let Some(error) = &facts.parse_error {
                anyhow::bail!(
                    "failed to parse imported setup helper {}: {error}",
                    file.display()
                );
            }
            let Some(dynamic) = facts.dynamic_imports.as_ref() else {
                anyhow::bail!(
                    "missing dynamic import facts for imported setup helper {}",
                    file.display()
                );
            };
            Ok(resolve_mock_specifiers(
                &dynamic.mock_specifiers,
                file,
                request.resolver,
                Some(request.graph_files),
            ))
        },
    )
}
