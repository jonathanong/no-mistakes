use super::super::RULE_ID;
use super::{config, setup_mocks};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::ts_resolver::ScopedImportResolver;
use crate::codebase::ts_source::has_disable_file_comment;
use crate::fx::FxHashMap;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct PreparedTest {
    pub(super) file: PathBuf,
    pub(super) contexts: Vec<Vec<PathBuf>>,
    pub(super) setup_needed: bool,
}

pub(super) struct Prepared {
    pub(super) tests: Vec<PreparedTest>,
    pub(super) mocks: FxHashMap<Vec<PathBuf>, HashSet<PathBuf>>,
}

pub(super) struct Request<'a> {
    pub(super) root: &'a Path,
    pub(super) test_files: Vec<PathBuf>,
    pub(super) setup_data: &'a [config::ConfigSetupData],
    pub(super) selected: config::SelectedRunners,
    pub(super) resolver: &'a ScopedImportResolver<'a>,
    pub(super) graph: &'a DepGraph,
    pub(super) graph_files: &'a GraphFiles,
    pub(super) visible_files: &'a crate::fx::PathSet,
    pub(super) shared: &'a CheckFactMap,
    pub(super) defer_suppression: bool,
}

pub(super) fn prepare(request: Request<'_>) -> Result<Prepared> {
    let tests = request
        .test_files
        .into_iter()
        .map(|file| {
            let rel_path = crate::codebase::ts_source::relative_slash_path(request.root, &file);
            PreparedTest {
                contexts: config::setup_contexts_for_test_precomputed(
                    &rel_path,
                    request.setup_data,
                    request.selected,
                ),
                setup_needed: needs_setup_mocks(request.shared, &file, request.defer_suppression),
                file,
            }
        })
        .collect::<Vec<_>>();
    let mut unique = tests
        .iter()
        .filter(|test| test.setup_needed)
        .flat_map(|test| test.contexts.iter().cloned())
        .collect::<Vec<_>>();
    unique.sort();
    unique.dedup();
    // Each distinct setup closure is independent of the test that matched it.
    // Warm all graph traversals before the parallel per-test loop.
    let pairs = unique
        .into_par_iter()
        .map(|files| {
            let mocks = setup_mocks::with_facts(setup_mocks::Request {
                setup_files: &files,
                resolver: request.resolver,
                graph: request.graph,
                graph_files: request.graph_files,
                visible_files: request.visible_files,
                shared: request.shared,
            })?;
            Ok((files, mocks))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Prepared {
        tests,
        mocks: pairs.into_iter().collect(),
    })
}

fn needs_setup_mocks(shared: &CheckFactMap, file: &Path, defer_suppression: bool) -> bool {
    let Some(facts) = shared.ts.get(file) else {
        return false;
    };
    let Some(source) = facts.source.as_deref() else {
        return false;
    };
    let disabled = has_disable_file_comment(source, RULE_ID);
    if disabled && (facts.parse_error.is_some() || !defer_suppression) {
        return false;
    }
    facts.parse_error.is_none() && facts.dynamic_imports.is_some()
}
