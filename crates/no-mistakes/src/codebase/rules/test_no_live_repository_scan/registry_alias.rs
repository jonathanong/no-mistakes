//! Which function a registry member names, and whether that function scans.
use super::super::super::call_policy::site_source_node;
use super::super::config::Enumerator;
use super::super::enumerators::scan_of;
use super::home::{self, Home, Locate};
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::{CallTraversal, DepGraph, NodeId, ResolvedCallSite};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_map, FxHashMap};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

pub(super) type CallerIndex = FxHashMap<PathBuf, FxHashMap<CallableId, NodeId>>;

pub(super) struct Reach<'a> {
    graph: &'a DepGraph,
    scanners: crate::fx::FxHashSet<NodeId>,
    memo: RefCell<FxHashMap<NodeId, bool>>,
}

impl<'a> Reach<'a> {
    pub(super) fn new(graph: &'a DepGraph, enumerators: &[Enumerator]) -> Self {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("test-no-live-repository-scan.registry.scan-index-builds", 1);
        }
        let scanners = graph
            .resolved_call_sites()
            .iter()
            .filter(|site| scan_of(site, enumerators).is_some())
            .map(site_source_node)
            .collect();
        Self {
            graph,
            scanners,
            memo: RefCell::new(fx_map()),
        }
    }

    pub(super) fn reaches(&self, node: &NodeId) -> bool {
        if let Some(known) = self.memo.borrow().get(node).copied() {
            return known;
        }
        let sources = self.graph.call_source_nodes(
            std::slice::from_ref(node),
            CallTraversal::Transitive,
            None,
        );
        let found = sources.iter().any(|source| self.scanners.contains(source));
        self.memo.borrow_mut().insert(node.clone(), found);
        found
    }
}

pub(super) fn caller_nodes(graph: &DepGraph) -> CallerIndex {
    let mut index: CallerIndex = fx_map();
    for site in graph.resolved_call_sites() {
        let Some(id) = site.caller_id else {
            continue;
        };
        index
            .entry(site.file.clone())
            .or_default()
            .entry(id)
            .or_insert_with(|| site_source_node(site));
    }
    index
}

pub(super) fn function_node(
    locate: &Locate<'_>,
    site: &ResolvedCallSite,
    object: &str,
    key: &str,
    callers: &CallerIndex,
) -> Option<NodeId> {
    home::homes(locate, site, object)
        .into_iter()
        .find_map(|home| match home {
            Home::Object(file, local) => object_member(locate, callers, &file, &local, key),
            Home::Namespace(file) => namespace_member(locate, callers, &file, key),
        })
}

pub(super) fn registry_scans(
    locate: &Locate<'_>,
    reach: &Reach<'_>,
    site: &ResolvedCallSite,
    object: &str,
    callers: &CallerIndex,
) -> bool {
    home::homes(locate, site, object)
        .into_iter()
        .any(|home| match home {
            Home::Object(file, local) => locate
                .facts
                .get_ts_facts(&file)
                .is_some_and(|ts| scans_member(reach, ts, &file, &local, callers)),
            Home::Namespace(file) => home::function_exports(locate, &file)
                .into_iter()
                .any(|(file, name)| namespace_scans(locate, reach, callers, &file, &name)),
        })
}

fn object_member(
    locate: &Locate<'_>,
    callers: &CallerIndex,
    file: &Path,
    local: &str,
    key: &str,
) -> Option<NodeId> {
    let ts = locate.facts.get_ts_facts(file)?;
    let name = terminal_alias(ts, &format!("{local}.{key}"))?;
    callable(callers, file, ts, &name)
}

fn namespace_member(
    locate: &Locate<'_>,
    callers: &CallerIndex,
    file: &Path,
    key: &str,
) -> Option<NodeId> {
    let (file, local) = home::binding_local(locate, file, key, &mut Vec::new())?;
    let ts = locate.facts.get_ts_facts(&file)?;
    let name = if ts
        .callable_scope_ids
        .iter()
        .any(|(_, scope)| scope == &local)
    {
        local
    } else {
        terminal_alias(ts, &local)?
    };
    callable(callers, &file, ts, &name)
}

fn namespace_scans(
    locate: &Locate<'_>,
    reach: &Reach<'_>,
    callers: &CallerIndex,
    file: &Path,
    name: &str,
) -> bool {
    locate.facts.get_ts_facts(file).is_some_and(|ts| {
        callable(callers, file, ts, name).is_some_and(|node| reach.reaches(&node))
    })
}

fn scans_member(
    reach: &Reach<'_>,
    ts: &TsFileFacts,
    file: &Path,
    local: &str,
    callers: &CallerIndex,
) -> bool {
    let prefix = format!("{local}.");
    ts.callable_aliases.iter().any(|alias| {
        alias.scope_id.is_none()
            && alias.invalidated_at.is_none()
            && alias.local.starts_with(&prefix)
            && terminal_alias(ts, &alias.local)
                .and_then(|name| callable(callers, file, ts, &name))
                .is_some_and(|node| reach.reaches(&node))
    })
}

fn terminal_alias(ts: &TsFileFacts, local: &str) -> Option<String> {
    let alias = ts.callable_aliases.iter().find(|alias| {
        alias.scope_id.is_none() && alias.invalidated_at.is_none() && alias.local == local
    })?;
    (!alias.target.contains('.')
        && ts
            .callable_scope_ids
            .iter()
            .any(|(_, scope)| scope == &alias.target))
    .then(|| alias.target.clone())
}

fn callable(callers: &CallerIndex, file: &Path, ts: &TsFileFacts, name: &str) -> Option<NodeId> {
    let id = ts
        .callable_scope_ids
        .iter()
        .find(|(_, scope)| scope == name)
        .map(|(id, _)| *id)?;
    callers.get(file).and_then(|ids| ids.get(&id)).cloned()
}
