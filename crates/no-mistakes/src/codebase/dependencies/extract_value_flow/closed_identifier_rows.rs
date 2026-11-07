use super::*;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::Expression;

#[derive(Default)]
pub(crate) struct State {
    pub(super) named: FxHashMap<usize, FxHashMap<String, ReadUse>>,
    pub(super) lookups: FxHashMap<usize, FxHashSet<String>>,
    pub(super) direct: FxHashMap<u32, ReadUse>,
    pub(super) allowed: FxHashSet<u32>,
    pub(crate) writing: bool,
}

#[derive(Default)]
pub(super) struct ReadUse {
    pub(super) fields: FxHashSet<String>,
    pub(super) invalid: bool,
}

pub(super) fn note_binding(
    collector: &mut ImportCollector,
    name: &str,
    scope: usize,
    init: &Expression<'_>,
) {
    let Some(call) = plain_call(init) else {
        return;
    };
    if module_callee(collector, &call.callee).is_some() {
        collector
            .metadata_rows
            .named
            .entry(scope)
            .or_default()
            .entry(name.to_string())
            .or_default();
    }
}

pub(super) fn note_lookup(collector: &mut ImportCollector, name: &str, scope: usize) {
    collector
        .metadata_rows
        .named
        .entry(scope)
        .or_default()
        .entry(name.to_string())
        .or_default();
    collector
        .metadata_rows
        .lookups
        .entry(scope)
        .or_default()
        .insert(name.to_string());
}

pub(super) fn module_callee(
    collector: &ImportCollector,
    expr: &Expression<'_>,
) -> Option<CallableId> {
    let Expression::Identifier(identifier) = crate::codebase::ts_source::unwrap_ts_wrappers(expr)
    else {
        return None;
    };
    (collector.callee_binding_scope(identifier.name.as_str()) == Some(0))
        .then(|| collector.callable_binding_at(0, identifier.name.as_str()))
        .flatten()
}

#[path = "closed_identifier_row_callers.rs"]
mod callers;
#[path = "closed_identifier_row_candidates.rs"]
mod candidates;
#[path = "closed_identifier_row_finalize.rs"]
pub(super) mod finalize;
#[path = "closed_identifier_row_index.rs"]
mod index;
#[path = "closed_identifier_row_reads.rs"]
mod reads;
#[path = "closed_identifier_row_returns.rs"]
mod returns;
