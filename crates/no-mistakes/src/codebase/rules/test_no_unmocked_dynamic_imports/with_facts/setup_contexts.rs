use super::super::RULE_ID;
use super::{config, setup_mocks};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphFiles};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_resolver::ScopedImportResolver;
use crate::codebase::ts_source::has_disable_file_comment;
use crate::config::v2::Unanalyzable;
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
    pub(super) diagnostics: AnalysisDiagnostics,
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
    pub(super) low: bool,
    pub(super) unanalyzable: Unanalyzable,
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
    let mut pairs = unique
        .into_par_iter()
        .map(|files| {
            let outcome = setup_mocks::with_facts(setup_mocks::Request {
                root: request.root,
                setup_files: &files,
                resolver: request.resolver,
                graph: request.graph,
                graph_files: request.graph_files,
                visible_files: request.visible_files,
                shared: request.shared,
                low: request.low,
                unanalyzable: request.unanalyzable,
            })?;
            Ok((files, outcome))
        })
        .collect::<Result<Vec<_>>>()?;
    pairs.sort_by(|left, right| left.0.cmp(&right.0));
    let mut diagnostics = AnalysisDiagnostics::default();
    let mocks = pairs
        .into_iter()
        .map(|(files, outcome)| {
            diagnostics.findings.extend(outcome.diagnostics.findings);
            diagnostics
                .advisories
                .extend(outcome.diagnostics.advisories);
            (files, outcome.mocks)
        })
        .collect();
    Ok(Prepared {
        tests,
        mocks,
        diagnostics,
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
