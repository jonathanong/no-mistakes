//! A const object of functions reached only through a member call.
#[path = "registry_alias.rs"]
mod alias;
#[path = "registry_home.rs"]
mod home;
#[path = "registry_resolve.rs"]
mod resolve;

use super::super::call_policy::site_source_node;
use super::super::path_filter::RulePathFilter;
use super::config::Enumerator;
use super::owned::path_provenance;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{
    CallTraversal, DepGraph, ResolvedCallSite, TsFactLookup,
};
use crate::codebase::ts_source::module_facts::arguments::is_name;
use crate::fx::PathSet;
use crate::perf_trace::trace;
use std::cell::OnceCell;
use std::path::PathBuf;

pub(super) fn apply<'a>(
    graph: &'a DepGraph,
    facts: &dyn TsFactLookup,
    scope: &mut super::reachable::Scope,
    filter: &RulePathFilter,
    enumerators: &[Enumerator],
    resolution: &super::Resolution<'_>,
    graph_files: &[PathBuf],
) -> Vec<(&'a ResolvedCallSite, String)> {
    let callers = OnceCell::new();
    let reach = OnceCell::new();
    let files = OnceCell::new();
    let locate = OnceCell::new();
    let mut reached = Vec::new();
    let mut unresolved = Vec::new();
    for site in graph.resolved_call_sites() {
        if site.invocation != InvocationKind::Call
            || site.target_node.is_some()
            || !scope.sources.contains(&site_source_node(site))
            || !filter.is_match(&site.file)
        {
            continue;
        }
        let Some((object, key, label)) = dispatch(facts, site) else {
            continue;
        };
        let callers = callers.get_or_init(|| {
            trace("test-no-live-repository-scan.registry.caller-index", || {
                alias::caller_nodes(graph)
            })
        });
        let locate = locate.get_or_init(|| {
            let files = files.get_or_init(|| graph_files.iter().cloned().collect::<PathSet>());
            home::Locate {
                graph,
                facts,
                resolution,
                visible: files,
                resolver: resolve::RegistryResolver::prepare(resolution, files),
            }
        });
        let resolved = key
            .as_deref()
            .and_then(|key| alias::function_node(locate, site, &object, key, callers));
        if let Some(node) = resolved {
            reached.push(node);
            continue;
        }
        if key.is_none() {
            let reach = reach.get_or_init(|| {
                trace("test-no-live-repository-scan.registry.scan-index", || {
                    alias::Reach::new(graph, enumerators)
                })
            });
            if alias::registry_scans(locate, reach, site, &object, callers) {
                unresolved.push((site, label));
            }
        }
    }
    if !reached.is_empty() {
        let extra = graph.call_source_nodes(&reached, CallTraversal::Transitive, None);
        scope.sources.extend(extra);
    }
    unresolved
}

/// `object.member`, or a computed call recorded while the file was parsed.
fn dispatch(
    facts: &dyn TsFactLookup,
    site: &ResolvedCallSite,
) -> Option<(String, Option<String>, String)> {
    if let Some((object, key)) = static_member(&site.source_callee) {
        return Some((object, key, String::new()));
    }
    let provenance = path_provenance(facts.get_ts_facts(&site.file)?)?;
    provenance
        .registry_dispatches
        .iter()
        .find(|dispatch| dispatch.0 == site.offset)
        .map(|dispatch| (dispatch.1.clone(), dispatch.2.clone(), dispatch.3.clone()))
}

fn static_member(callee: &str) -> Option<(String, Option<String>)> {
    let (object, key) = callee.split_once('.')?;
    (is_name(object) && is_name(key)).then(|| (object.to_string(), Some(key.to_string())))
}

#[cfg(test)]
#[path = "registry_coverage_tests.rs"]
mod coverage_tests;
