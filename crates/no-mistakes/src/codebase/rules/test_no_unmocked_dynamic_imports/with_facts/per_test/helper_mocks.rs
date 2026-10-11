use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphFiles, NodeId};
use crate::codebase::rules::test_no_unmocked_dynamic_imports::resolve_mock_specifiers;
use crate::codebase::ts_resolver::ScopedImportResolver;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct Request<'a> {
    pub(super) test_file: &'a Path,
    pub(super) graph: &'a DepGraph,
    pub(super) graph_files: &'a GraphFiles,
    pub(super) visible_files: &'a crate::fx::PathSet,
    pub(super) resolver: &'a ScopedImportResolver<'a>,
    pub(super) shared: &'a CheckFactMap,
    pub(super) excluded: &'a HashSet<PathBuf>,
}

pub(super) fn collect(request: Request<'_>) -> HashSet<PathBuf> {
    let Request {
        test_file,
        graph,
        graph_files,
        visible_files,
        resolver,
        shared,
        excluded,
    } = request;
    // Mock registrations execute only when their helper is loaded statically.
    // A typed mock specifier's import(...) is a type carrier, not a helper edge.
    let allowed = [EdgeKind::Import, EdgeKind::WorkspaceImport].into();
    graph
        .deps_of_in_file_universe(
            &[NodeId::file(test_file)],
            None,
            Some(&allowed),
            visible_files,
        )
        .into_iter()
        .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
        .filter(|file| !excluded.contains(file))
        .filter_map(|file| {
            shared.ts.get(&file).and_then(|facts| {
                facts
                    .dynamic_imports
                    .as_ref()
                    .map(|dynamic| (file, dynamic))
            })
        })
        .flat_map(|(file, facts)| {
            resolve_mock_specifiers(&facts.mock_specifiers, &file, resolver, Some(graph_files))
        })
        .collect()
}
