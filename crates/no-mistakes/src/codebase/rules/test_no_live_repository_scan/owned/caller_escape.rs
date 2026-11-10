//! An escape of a function that reaches the enumerator blocks that enumerator.
use super::cause::{kind_of, Cause, CauseKind};
use super::Owners;
use crate::codebase::dependencies::graph::{NodeId, TsFactLookup};
use crate::codebase::rules::forbidden_calls::site_source_node;
use crate::fx::FxHashSet;

impl Owners<'_> {
    /// The enumerator itself is fine. A caller, or a function that reaches it, is not.
    pub(super) fn parameter_proven(&self, source: &NodeId) -> bool {
        let NodeId::Symbol { file, symbol, .. } = source else {
            return false;
        };
        !self.escaped.contains(file, symbol)
            && !super::symbol_block::blocks(&self.consumed, file, symbol)
    }

    pub(super) fn caller_blocked(
        &self,
        source: &NodeId,
        facts: &dyn TsFactLookup,
    ) -> Option<Cause> {
        let mut seen = FxHashSet::default();
        let mut stack = vec![source.clone()];
        seen.insert(source.clone());
        while let Some(node) = stack.pop() {
            let Some(callers) = self.callers.get(&node) else {
                continue;
            };
            for caller in callers {
                let origin = site_source_node(caller);
                if origin == node || !seen.insert(origin.clone()) {
                    continue;
                }
                if let Some(cause) = symbol_cause(self, &origin, facts) {
                    return Some(cause);
                }
                stack.push(origin);
            }
        }
        None
    }
}

fn symbol_cause(owners: &Owners<'_>, source: &NodeId, facts: &dyn TsFactLookup) -> Option<Cause> {
    let NodeId::Symbol { file, symbol, .. } = source else {
        return None;
    };
    if owners.escaped.contains(file, symbol) {
        let found = owners.escaped.reference(file, symbol);
        let (path, line, name) = found.unwrap_or_else(|| {
            let (path, line) = super::cause::function_site(facts, source);
            (path, line, symbol.to_string())
        });
        return Some(Cause::new(CauseKind::NonDirectReference, path, line, name));
    }
    if !super::symbol_block::blocks(&owners.consumed, file, symbol) {
        return None;
    }
    if let Some(hit) = super::symbol_block::route(&owners.consumed, file, symbol) {
        return Some(Cause::new(
            kind_of(hit.kind),
            hit.file.clone(),
            hit.line,
            hit.detail.clone(),
        ));
    }
    Some(super::route_cause::function_fallback(
        &owners.callers,
        source,
        facts,
        None,
    ))
}

#[cfg(test)]
#[path = "caller_escape/tests.rs"]
mod tests;
