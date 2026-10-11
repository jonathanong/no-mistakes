use super::super::resolve_mock_specifiers;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

pub(super) struct Request<'a> {
    pub(super) setup_files: &'a [PathBuf],
    pub(super) resolver: &'a crate::codebase::ts_resolver::ScopedImportResolver<'a>,
    pub(super) graph: &'a DepGraph,
    pub(super) graph_files: &'a GraphFiles,
    pub(super) visible_files: &'a crate::fx::PathSet,
    pub(super) shared: &'a CheckFactMap,
}

pub(super) fn with_facts(request: Request<'_>) -> Result<HashSet<PathBuf>> {
    from_group(request.setup_files, &request)
}

fn from_group(files: &[PathBuf], request: &Request<'_>) -> Result<HashSet<PathBuf>> {
    let mut mocks = HashSet::new();
    for setup in files {
        let Some(file_facts) = request.shared.ts.get(setup) else {
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
            setup,
            request.resolver,
            Some(request.graph_files),
        ));
        mocks.extend(super::per_test::helper_mocks::collect_strict(
            super::per_test::helper_mocks::Request {
                test_file: setup,
                graph: request.graph,
                graph_files: request.graph_files,
                visible_files: request.visible_files,
                resolver: request.resolver,
                shared: request.shared,
                excluded: &mocks,
            },
        )?);
    }
    Ok(mocks)
}
