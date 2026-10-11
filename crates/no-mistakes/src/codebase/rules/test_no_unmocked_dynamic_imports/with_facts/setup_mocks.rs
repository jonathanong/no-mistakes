use super::super::{config, resolve_mock_specifiers};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct Request<'a> {
    pub(super) root: &'a Path,
    pub(super) setup_data: &'a [config::ConfigSetupData],
    pub(super) test_file: &'a Path,
    pub(super) resolver: &'a crate::codebase::ts_resolver::ScopedImportResolver<'a>,
    pub(super) graph: &'a DepGraph,
    pub(super) graph_files: &'a GraphFiles,
    pub(super) visible_files: &'a crate::fx::PathSet,
    pub(super) shared: &'a CheckFactMap,
}

pub(super) fn with_facts(request: Request<'_>) -> Result<HashSet<PathBuf>> {
    let Request {
        root,
        setup_data,
        test_file,
        resolver,
        graph,
        graph_files,
        visible_files,
        shared,
    } = request;
    let mut mocks = HashSet::new();
    let rel_path = crate::codebase::ts_source::relative_slash_path(root, test_file);
    for setup in config::setup_files_for_test_precomputed(&rel_path, setup_data) {
        let Some(file_facts) = shared.ts.get(&setup) else {
            anyhow::bail!("missing shared facts for {}", setup.display());
        };
        if let Some(error) = &file_facts.parse_error {
            anyhow::bail!("failed to parse {}: {error}", setup.display());
        }
        let Some(facts) = file_facts.dynamic_imports.as_ref() else {
            anyhow::bail!("missing dynamic import facts for {}", setup.display());
        };
        mocks.extend(resolve_mock_specifiers(
            &facts.mock_specifiers,
            &setup,
            resolver,
            Some(graph_files),
        ));
        mocks.extend(super::per_test::helper_mocks::collect(
            super::per_test::helper_mocks::Request {
                test_file: &setup,
                graph,
                graph_files,
                visible_files,
                resolver,
                shared,
                excluded: &mocks,
            },
        ));
    }
    Ok(mocks)
}
