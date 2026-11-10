//! Resolve a parameter or env-var root through the callers that reach it.
//!
//! Recall first: an unreadable module that a test root can use to reach the
//! function keeps the finding. A module outside that closure does not.
mod caller_escape;
mod cause;
mod closure;
mod closure_path;
mod consumers;
#[cfg(test)]
#[path = "owned/coverage/tests.rs"]
mod coverage;
mod entry_memo;
pub(super) mod env;
mod env_query;
mod escapes;
mod imports;
mod local_env;
mod parameter_cause;
mod parameters;
mod route_cause;
mod route_index;
pub(super) mod setups;
mod symbol_block;
pub(super) use cause::{Cause, Ownership};
pub(crate) use consumers::Resolution;

use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{DepGraph, NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
use crate::fx::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::path::PathBuf;

/// Path facts of a file whose parse is trustworthy.
pub(super) fn path_provenance(facts: &TsFileFacts) -> Option<&PathProvenance> {
    let parsed = facts.operational_error.is_none()
        && facts.parse_error.is_none()
        && !facts.fatal_parse_error;
    let module = facts.module_bindings.as_deref()?;
    let provenance = module.path_provenance.as_deref()?;
    let understood = module.diagnostics.iter().all(|diagnostic| {
        diagnostic.kind == "unsupported"
            && diagnostic.message == "Non-literal dynamic import specifier"
            && diagnostic.span.as_ref().is_some_and(|span| {
                provenance
                    .resolved_import_spans
                    .contains(&(span.start, span.end))
            })
    });
    (parsed && understood).then_some(provenance)
}

pub(super) struct Owners<'g> {
    /// Reachable call sites by the callable they invoke.
    callers: FxHashMap<NodeId, Vec<&'g ResolvedCallSite>>,
    env: env::EnvWrites,
    imported: imports::ImportedWrites<'g>,
    setups: setups::ConfiguredSetups,
    fixture_dirs: Vec<String>,
    nodes: FxHashMap<PathBuf, FxHashMap<u32, NodeId>>,
    /// Functions referenced other than as a direct callee in reachable code.
    escaped: escapes::Escapes,
    /// Files consumed other than by named static imports used only as calls.
    consumed: consumers::Consumed,
    graph: &'g DepGraph,
    /// Test files, setupFiles, globalSetup, and their import closure.
    test_closure: FxHashSet<PathBuf>,
    /// Test files, `setupFiles`, and `globalSetup` before the closure expands.
    gate_roots: Vec<PathBuf>,
    /// Per enumerating module: whether an unreadable file can reach it.
    reached: RefCell<FxHashMap<PathBuf, bool>>,
    /// Caller-entry walks memoized by the symbol they start from.
    entry_memo: RefCell<entry_memo::EntryMemo<'g>>,
}

impl<'g> Owners<'g> {
    pub(super) fn new(
        graph: &'g DepGraph,
        scope: &super::reachable::Scope,
        facts: &dyn TsFactLookup,
        graph_files: &[PathBuf],
        resolution: &Resolution<'_>,
        setups: setups::ConfiguredSetups,
        environment: Environment<'g>,
    ) -> Self {
        let mut callers: FxHashMap<NodeId, Vec<&ResolvedCallSite>> = FxHashMap::default();
        let mut nodes: FxHashMap<PathBuf, FxHashMap<u32, NodeId>> = FxHashMap::default();
        for site in graph.resolved_call_sites() {
            for node in [Some(site_source_node(site)), site.target_node.clone()]
                .into_iter()
                .flatten()
            {
                if let NodeId::Symbol {
                    file,
                    callable_id: Some(id),
                    ..
                } = &node
                {
                    nodes
                        .entry(file.to_path_buf())
                        .or_default()
                        .insert(id.0, node);
                }
            }
            if site.invocation == InvocationKind::Call
                && scope.sources.contains(&site_source_node(site))
            {
                if let Some(target) = &site.target_node {
                    callers.entry(target.clone()).or_default().push(site);
                }
            }
        }
        let relevant_files = &scope.files;
        let escaped = escapes::Escapes::new(facts, graph_files, relevant_files);
        let consumed =
            consumers::Consumed::new(graph, facts, graph_files, relevant_files, resolution);
        Self {
            callers,
            nodes,
            graph,
            test_closure: scope.test_closure.clone(),
            gate_roots: scope.gate_roots.clone(),
            reached: RefCell::new(FxHashMap::default()),
            entry_memo: RefCell::new(entry_memo::EntryMemo::default()),
            imported: imports::ImportedWrites::new(graph),
            env: env::EnvWrites::new(
                facts,
                graph_files,
                relevant_files,
                environment.root,
                environment.config,
                &environment.fixture_dirs,
                scope.config_files.as_ref(),
            ),
            setups,
            fixture_dirs: environment.fixture_dirs,
            escaped,
            consumed,
        }
    }

    /// A named function, referenced only as a direct callee, in a module
    /// every consumer imports by name and only calls.
    fn proven_target(&self, source: &NodeId) -> bool {
        let NodeId::Symbol { file, symbol, .. } = source else {
            return false;
        };
        let name: &str = symbol;
        !self.escaped.contains(file, name) && !self.consumed.contains(file)
    }

    pub(super) fn report_scan_stats(&self) {
        if std::env::var_os("NO_MISTAKES_SCAN_RESOLVE_DEBUG").is_none() {
            return;
        }
        let entries = self.entry_memo.borrow();
        eprintln!(
            "scan-resolve entries walks={} hits={}",
            entries.walks, entries.hits
        );
        self.consumed.report_targeting();
    }
}

pub(super) struct Environment<'a> {
    pub(super) root: &'a std::path::Path,
    pub(super) config: &'a crate::config::v2::NoMistakesConfig,
    pub(super) fixture_dirs: Vec<String>,
}
