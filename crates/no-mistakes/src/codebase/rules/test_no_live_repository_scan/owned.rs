//! Resolve a parameter or env-var root through the callers that reach it.
//!
//! Recall first: any caller, write, or file this pass cannot read keeps the finding.
mod consumers;
pub(super) mod env;
pub(crate) use consumers::Resolution;

use super::provenance::{classify, Verdict};
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{DepGraph, NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::{PathOrigin, PathProvenance};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;

/// Path facts of a file whose parse is trustworthy.
pub(super) fn path_provenance(facts: &TsFileFacts) -> Option<&PathProvenance> {
    let parsed = facts.operational_error.is_none()
        && facts.parse_error.is_none()
        && !facts.fatal_parse_error;
    let module = facts.module_bindings.as_deref()?;
    (parsed && module.diagnostics.is_empty())
        .then_some(module.path_provenance.as_deref())
        .flatten()
}

pub(super) struct Owners<'g> {
    /// Reachable call sites by the callable they invoke.
    callers: FxHashMap<NodeId, Vec<&'g ResolvedCallSite>>,
    env: env::EnvWrites,
    /// Names of functions referenced other than as a direct callee, anywhere.
    escaped: FxHashSet<String>,
    /// Files consumed other than by named static imports used only as calls.
    consumed: consumers::Consumed,
    /// A file whose facts failed, or a module load that cannot be read, may hide a consumer.
    escape_unknown: bool,
}

impl<'g> Owners<'g> {
    pub(super) fn new(
        graph: &'g DepGraph,
        reachable: &FxHashSet<NodeId>,
        facts: &dyn TsFactLookup,
        graph_files: &[PathBuf],
        resolution: &Resolution<'_>,
        root: &std::path::Path,
        config: &crate::config::v2::NoMistakesConfig,
    ) -> Self {
        let mut callers: FxHashMap<NodeId, Vec<&ResolvedCallSite>> = FxHashMap::default();
        for site in graph.resolved_call_sites() {
            if site.invocation == InvocationKind::Call
                && reachable.contains(&site_source_node(site))
            {
                if let Some(target) = &site.target_node {
                    callers.entry(target.clone()).or_default().push(site);
                }
            }
        }
        let mut escaped = FxHashSet::default();
        let mut escape_unknown = false;
        for ts in graph_files
            .iter()
            .filter_map(|file| facts.get_ts_facts(file))
        {
            match path_provenance(ts) {
                Some(provenance) => {
                    escaped.extend(provenance.value_refs.iter().cloned());
                    escape_unknown |= provenance.modules_unknown;
                }
                None => escape_unknown |= ts.module_bindings.is_some(),
            }
        }
        let consumed = consumers::Consumed::new(facts, graph_files, resolution);
        escape_unknown |= consumed.unknown();
        Self {
            callers,
            env: env::EnvWrites::new(facts, graph_files, root, config),
            escaped,
            consumed,
            escape_unknown,
        }
    }

    /// True when every route into the enumeration provably supplies an owned root.
    pub(super) fn owned(
        &self,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> bool {
        // An owned root is only owned below itself; `..` climbs out of it.
        if origin.segments.iter().any(|segment| segment == "..") {
            return false;
        }
        match (&origin.env, origin.param) {
            (Some(name), _) => self.env_owned(name, site, facts),
            (None, Some(index)) => self.param_owned(index, site, facts, fixture_dirs),
            (None, None) => false,
        }
    }

    /// A top-level named function, referenced only as a direct callee, in a module
    /// every consumer imports by name and only calls.
    fn proven_target(&self, source: &NodeId) -> bool {
        let NodeId::Symbol { file, symbol, .. } = source else {
            return false;
        };
        let name: &str = symbol;
        !name.contains('.') && !self.escaped.contains(name) && !self.consumed.contains(file)
    }

    /// Every caller passes an argument at `index` that is itself owned or bounded.
    /// A caller that omits it falls back to the default and keeps the finding.
    fn param_owned(
        &self,
        index: usize,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> bool {
        let source = site_source_node(site);
        let escapes = self.escape_unknown || !self.proven_target(&source);
        let Some(callers) = self.callers.get(&source).filter(|_| !escapes) else {
            return false;
        };
        callers.iter().all(|caller| {
            facts
                .get_ts_facts(&caller.file)
                .and_then(path_provenance)
                .and_then(|provenance| provenance.arg_paths.get(&caller.offset))
                .and_then(|paths| paths.iter().find(|(position, _)| *position == index))
                .is_some_and(|(_, origin)| classify(Some(origin), fixture_dirs) == Verdict::Allowed)
        })
    }

    /// Call sites that start a chain reaching `target`: sites whose own
    /// callable nothing calls. `None` when `target` itself has no caller.
    fn entries(&self, target: NodeId) -> Option<Vec<&'g ResolvedCallSite>> {
        let mut seen = FxHashSet::default();
        let mut stack = vec![target.clone()];
        let mut entries = Vec::new();
        while let Some(node) = stack.pop() {
            if !seen.insert(node.clone()) {
                continue;
            }
            let callers = self.callers.get(&node)?;
            for caller in callers {
                let source = site_source_node(caller);
                if self.callers.contains_key(&source) {
                    stack.push(source);
                } else {
                    entries.push(*caller);
                }
            }
        }
        entries.sort_by(|a, b| (&a.file, a.offset).cmp(&(&b.file, b.offset)));
        entries.dedup_by(|a, b| a.file == b.file && a.offset == b.offset);
        Some(entries)
    }

    fn env_owned(&self, name: &str, site: &ResolvedCallSite, facts: &dyn TsFactLookup) -> bool {
        if !self.env.all_owned(name) {
            return false;
        }
        let Some(entries) = self.entries(site_source_node(site)) else {
            return false;
        };
        !entries.is_empty()
            && entries.iter().all(|entry| {
                facts
                    .get_ts_facts(&entry.file)
                    .and_then(path_provenance)
                    .is_some_and(|provenance| env::dominated(provenance, name, entry.offset))
            })
    }
}
