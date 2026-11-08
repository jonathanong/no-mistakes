//! Resolve a parameter or env-var root through the callers that reach it.
//!
//! Recall first: any caller, write, or file this pass cannot read keeps the finding.
mod consumers;
pub(super) mod env;
mod env_query;
mod imports;
mod parameters;
mod setups;
pub(crate) use consumers::Resolution;

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
    setups: std::cell::OnceCell<setups::ConfiguredSetups>,
    root: &'g std::path::Path,
    catalog: Option<&'g crate::codebase::rules::PreparedVitestProjectCatalog>,
    nodes: FxHashMap<PathBuf, FxHashMap<u32, NodeId>>,
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
        let consumed = consumers::Consumed::new(graph, facts, graph_files, resolution);
        escape_unknown |= consumed.unknown();
        Self {
            callers,
            nodes,
            imported: imports::ImportedWrites::new(graph),
            env: env::EnvWrites::new(facts, graph_files, environment.root, environment.config),
            setups: std::cell::OnceCell::new(),
            root: environment.root,
            catalog: environment.vitest,
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
            (None, Some(index)) => self.param_owned(index, origin, site, facts, fixture_dirs),
            (None, None) => false,
        }
    }

    /// A named function, referenced only as a direct callee, in a module
    /// every consumer imports by name and only calls.
    fn proven_target(&self, source: &NodeId) -> bool {
        let NodeId::Symbol { file, symbol, .. } = source else {
            return false;
        };
        let name: &str = symbol;
        !self.escaped.contains(name)
            && !self
                .escaped
                .contains(name.rsplit('.').next().unwrap_or(name))
            && !self.consumed.contains(file)
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
}

pub(super) struct Environment<'a> {
    pub(super) root: &'a std::path::Path,
    pub(super) config: &'a crate::config::v2::NoMistakesConfig,
    pub(super) vitest: Option<&'a crate::codebase::rules::PreparedVitestProjectCatalog>,
}
