//! Resolve a parameter or env-var root through the callers that reach it.
//!
//! Recall first: any caller, write, or file this pass cannot read keeps the finding.
mod consumers;
pub(super) mod env;
mod env_query;
mod escapes;
mod imports;
mod local_env;
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
    setups: setups::ConfiguredSetups,
    fixture_dirs: Vec<String>,
    nodes: FxHashMap<PathBuf, FxHashMap<u32, NodeId>>,
    /// Functions referenced other than as a direct callee in reachable code.
    escaped: escapes::Escapes,
    /// Files consumed other than by named static imports used only as calls.
    consumed: consumers::Consumed,
    /// A file whose facts failed, or a module load that cannot be read, may hide a consumer.
    escape_unknown: bool,
}

impl<'g> Owners<'g> {
    pub(super) fn new(
        graph: &'g DepGraph,
        reachable: &FxHashSet<NodeId>,
        evidence_files: &FxHashSet<PathBuf>,
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
        let mut relevant_files = evidence_files.clone();
        let setups = setups::ConfiguredSetups::new(environment.root, environment.vitest);
        if let Some(setup_files) = setups.setup_files() {
            let setup_files: Vec<PathBuf> = setup_files.into_iter().collect();
            relevant_files.extend(super::reachable::module_files(graph, &setup_files));
        } else {
            relevant_files.extend(graph_files.iter().cloned());
        }
        let escaped = escapes::Escapes::new(facts, graph_files, &relevant_files);
        let consumed =
            consumers::Consumed::new(graph, facts, graph_files, &relevant_files, resolution);
        let escape_unknown = escaped.unknown() || consumed.unknown();
        Self {
            callers,
            nodes,
            imported: imports::ImportedWrites::new(graph),
            env: env::EnvWrites::new(
                facts,
                graph_files,
                &relevant_files,
                environment.root,
                environment.config,
                &environment.fixture_dirs,
            ),
            setups,
            fixture_dirs: environment.fixture_dirs,
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
        if origin.segments.iter().any(|segment| segment == "..")
            && (origin.param.is_some()
                || origin.env.is_some()
                || origin.anchor
                    == crate::codebase::ts_source::module_facts::arguments::PathAnchor::Temp)
        {
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
        !self.escaped.contains(file, name) && !self.consumed.contains(file)
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
    pub(super) fixture_dirs: Vec<String>,
}
