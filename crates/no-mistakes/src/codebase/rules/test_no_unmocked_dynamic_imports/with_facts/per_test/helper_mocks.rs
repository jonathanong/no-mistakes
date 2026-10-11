use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphFiles, NodeId};
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

pub(in super::super) fn collect(request: Request<'_>) -> HashSet<PathBuf> {
    let mut mocks = HashSet::new();
    for file in helper_files(&request) {
        if let Some(dynamic) = request
            .shared
            .ts
            .get(&file)
            .and_then(|facts| facts.dynamic_imports.as_ref())
        {
            mocks.extend(resolve_mock_specifiers(
                &dynamic.mock_specifiers,
                &file,
                request.resolver,
                Some(request.graph_files),
            ));
        }
    }
    mocks
}

pub(in super::super) fn collect_strict(request: Request<'_>) -> anyhow::Result<HashSet<PathBuf>> {
    let mut mocks = HashSet::new();
    for file in helper_files(&request) {
        let Some(facts) = request.shared.ts.get(&file) else {
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
        mocks.extend(resolve_mock_specifiers(
            &dynamic.mock_specifiers,
            &file,
            request.resolver,
            Some(request.graph_files),
        ));
    }
    Ok(mocks)
}

fn helper_files(request: &Request<'_>) -> Vec<PathBuf> {
    let Request {
        test_file,
        graph,
        visible_files,
        excluded,
        ..
    } = request;
    // Mock registrations execute only when their helper is loaded statically.
    // A typed mock specifier's import(...) is a type carrier, not a helper edge.
    let allowed = [EdgeKind::Import, EdgeKind::WorkspaceImport].into();
    graph
        .deps_of_in_file_universe_excluding_files(
            &[NodeId::file(test_file)],
            Some(&allowed),
            visible_files,
            excluded,
        )
        .into_iter()
        .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
        .filter(|file| !excluded.contains(file))
        .filter(|file| crate::codebase::dependencies::extract::is_indexable(file))
        .collect()
}
