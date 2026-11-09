//! ECMAScript and Node built-in calls which cannot reach a repository assertion.
//! The model and its limits are documented in docs/rules/coverage-producer-assertion-owner.md.
use super::builtin_tables::*;
use super::*;
use crate::codebase::dependencies::extract::{InvocationKind, ValueCall, ValueExpression};
use crate::codebase::dependencies::graph::ResolvedCallTarget;
#[path = "builtins/call_index.rs"]
mod call_index;
#[path = "builtins/timers.rs"]
mod timers;
pub(super) use call_index::CallIndex;

/// A built-in call and the callbacks it invokes.
#[derive(Default)]
pub(super) struct Callbacks<'a> {
    /// Resolved callback occurrences in the same callable which the built-in executes.
    pub executed: Vec<&'a ResolvedCallSite>,
    /// Callback arguments the graph could not resolve, formatted for messages.
    pub unresolved: Vec<String>,
}

/// Classify an unresolved call as a non-dispatching built-in; `None` when it is not one.
/// Callback arguments of callback-invoking built-ins are returned, never dropped.
pub(super) fn classify<'a>(
    site: &ResolvedCallSite,
    node_sites: &[&'a ResolvedCallSite],
    calls: &CallIndex<'_>,
    facts: &dyn TsFactLookup,
) -> Option<Callbacks<'a>> {
    if site.invocation != InvocationKind::Call
        || !matches!(
            site.target,
            ResolvedCallTarget::Unknown | ResolvedCallTarget::Global { .. }
        )
    {
        return None;
    }
    let mut out = Callbacks::default();
    if !site.source_callee.contains('.') {
        if GLOBAL_FUNCTIONS.contains(&site.source_callee.as_str()) {
            return Some(out);
        }
        if TIMERS.contains(&site.source_callee.as_str())
            && matches!(&site.target, ResolvedCallTarget::Global { name } if name == &site.source_callee)
        {
            return Some(timers::classify(site, node_sites, calls));
        }
        return None;
    }
    let selected = calls
        .at(site)
        .iter()
        .copied()
        .filter(|call| call.source_callee == site.source_callee)
        .collect::<Vec<_>>();
    let file = facts.get_ts_facts(&site.file)?;
    let mut native = false;
    let mut leaf = false;
    let mut missing = false;
    for call in &selected {
        if file
            .known_nonbuiltin_receiver_calls
            .contains(&(call.offset, call.end))
        {
            return None;
        }
        match file.builtin_callback_calls.get(&(call.offset, call.end)) {
            Some(Some(positions)) => {
                native = true;
                collect_arguments(Some(*call), positions, node_sites, &mut out);
            }
            Some(None) => leaf = true,
            None => missing = true,
        }
    }
    if missing && (native || leaf) {
        return None;
    }
    if native {
        return Some(out);
    }
    let (receiver, method) = site.source_callee.rsplit_once('.')?;
    (leaf && (GLOBAL_OBJECTS.contains(&receiver) || METHODS.contains(&method))
        || missing
            && !selected.is_empty()
            && matches!(site.target, ResolvedCallTarget::Unknown)
            && METHODS.contains(&method))
    .then_some(out)
}

fn collect_arguments<'a>(
    call: Option<&ValueCall>,
    positions: &[usize],
    node_sites: &[&'a ResolvedCallSite],
    out: &mut Callbacks<'a>,
) {
    for position in positions {
        if call.is_some_and(|call| call.spread_at.is_some_and(|spread| *position >= spread)) {
            out.unresolved.push("spread arguments".to_string());
            break;
        }
        let Some(argument) = call.and_then(|call| call.arguments.get(*position)) else {
            continue;
        };
        let callback = |matches: &dyn Fn(&ResolvedCallSite) -> bool| {
            node_sites
                .iter()
                .copied()
                .find(|other| other.invocation == InvocationKind::Callback && matches(other))
        };
        match argument {
            ValueExpression::Callable(id) => match callback(&|other| other.offset == id.0) {
                Some(found) => out.executed.push(found),
                None => out.unresolved.push(format!("argument {}", position + 1)),
            },
            ValueExpression::Named { name, offset, .. } => {
                match callback(&|other| other.source_callee == *name && other.offset == *offset) {
                    Some(found) if !found.has_unknown_target() => out.executed.push(found),
                    _ => out.unresolved.push(name.clone()),
                }
            }
            ValueExpression::Undefined
            | ValueExpression::Null
            | ValueExpression::Object(..)
            | ValueExpression::Array(..) => {}
            _ => {
                out.unresolved.push(format!("argument {}", position + 1));
            }
        }
    }
}

/// Whether this candidate is a synchronous Node built-in module export.
pub(super) fn node_module(candidate: &KnownCallTargetRef<'_>) -> bool {
    matches!(candidate, KnownCallTargetRef::ModuleExport { specifier, repository_target: None, .. }
        if NODE_MODULES.contains(specifier))
}
