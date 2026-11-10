//! The route that made a parameter's callers unknowable.
use super::cause::{function_site, kind_of, Cause, CauseKind};
use super::setups::SetupGap;
use super::Owners;
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::fx::FxHashMap;

impl Owners<'_> {
    pub(super) fn route_cause(&self, source: &NodeId, facts: &dyn TsFactLookup) -> Cause {
        if !self.setups.known() {
            return setup_gap(self, source, facts);
        }
        if let Some(module) = source.as_file() {
            if self.closure_unknown(module) {
                return self.unread_cause(module, source, facts);
            }
        }
        local_route(self, source, facts)
    }
}

fn setup_gap(owners: &Owners<'_>, source: &NodeId, facts: &dyn TsFactLookup) -> Cause {
    match owners.setups.gap() {
        Some(SetupGap::Unresolved(holes)) if !holes.is_empty() => {
            let (file, line) = holes[0].clone();
            let extra = holes.len().saturating_sub(1) as u32;
            Cause::new(CauseKind::UnresolvedSetup, file, line, "").with_extra(extra)
        }
        Some(SetupGap::Unreadable(file)) => {
            Cause::new(CauseKind::UnreadableConfig, file.clone(), 1, "")
        }
        _ => {
            let (file, line) = function_site(facts, source);
            Cause::new(CauseKind::UnreadableConfig, file, line, "")
        }
    }
}

fn local_route(owners: &Owners<'_>, source: &NodeId, facts: &dyn TsFactLookup) -> Cause {
    let NodeId::Symbol { file, symbol, .. } = source else {
        return hole(CauseKind::NotNamedFunction, source, facts);
    };
    if owners.escaped.contains(file, symbol) {
        let found = owners.escaped.reference(file, symbol);
        let (path, line, name) = found.unwrap_or_else(|| named_site(facts, source, symbol));
        return Cause::new(CauseKind::NonDirectReference, path, line, name);
    }
    if super::symbol_block::blocks(&owners.consumed, file, symbol) {
        if let Some(hit) = super::symbol_block::route(&owners.consumed, file, symbol) {
            return Cause::new(
                kind_of(hit.kind),
                hit.file.clone(),
                hit.line,
                hit.detail.clone(),
            );
        }
    }
    function_fallback(&owners.callers, source, facts)
}

pub(super) fn function_fallback(
    callers: &FxHashMap<NodeId, Vec<&ResolvedCallSite>>,
    source: &NodeId,
    facts: &dyn TsFactLookup,
) -> Cause {
    let (file, line) = function_site(facts, source);
    let kind = if recursive_helper(callers, source) {
        CauseKind::RecursiveHelper
    } else {
        CauseKind::CallersNotProven
    };
    Cause::new(kind, file, line, "")
}

fn recursive_helper(callers: &FxHashMap<NodeId, Vec<&ResolvedCallSite>>, source: &NodeId) -> bool {
    if calls_self(callers, source) {
        return true;
    }
    callers.iter().any(|(target, sites)| {
        sites.iter().any(|site| site_source_node(site) == *source) && calls_self(callers, target)
    })
}

fn calls_self(callers: &FxHashMap<NodeId, Vec<&ResolvedCallSite>>, node: &NodeId) -> bool {
    callers
        .get(node)
        .is_some_and(|sites| sites.iter().any(|site| site_source_node(site) == *node))
}

fn hole(kind: CauseKind, source: &NodeId, facts: &dyn TsFactLookup) -> Cause {
    let (file, line) = function_site(facts, source);
    Cause::new(kind, file, line, "")
}

fn named_site(
    facts: &dyn TsFactLookup,
    source: &NodeId,
    name: &str,
) -> (std::path::PathBuf, u32, String) {
    let (path, line) = function_site(facts, source);
    (path, line, name.to_string())
}

#[cfg(test)]
#[path = "route_cause_tests.rs"]
mod route_cause_tests;
