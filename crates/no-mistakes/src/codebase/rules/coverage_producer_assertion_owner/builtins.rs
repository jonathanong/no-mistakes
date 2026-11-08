//! ECMAScript and Node built-in calls which cannot reach a repository assertion.
//! The model and its limits are documented in docs/rules/coverage-producer-assertion-owner.md.
use super::builtin_tables::*;
use super::*;
use crate::codebase::dependencies::extract::{InvocationKind, ValueExpression};
use crate::codebase::dependencies::graph::ResolvedCallTarget;

/// A built-in call and the callbacks it invokes.
#[derive(Default)]
pub(super) struct Callbacks<'a> {
    /// Resolved callback occurrences in the same callable which the built-in executes.
    pub executed: Vec<&'a ResolvedCallSite>,
    /// Callback arguments the graph could not resolve, formatted for messages.
    pub unresolved: Vec<String>,
}

fn positions(site: &ResolvedCallSite, facts: &dyn TsFactLookup) -> Option<&'static [usize]> {
    if !matches!(site.target, ResolvedCallTarget::Unknown)
        || site.invocation != InvocationKind::Call
    {
        return None;
    }
    let callee = site.source_callee.as_str();
    let Some((receiver, method)) = callee.rsplit_once('.') else {
        return GLOBAL_FUNCTIONS
            .contains(&callee)
            .then_some(&[][..])
            .or_else(|| TIMERS.contains(&callee).then_some(&[0][..]));
    };
    let proven = facts
        .get_ts_facts(&site.file)
        .is_some_and(|file| file.builtin_receiver_calls.contains(&site.offset));
    if !proven {
        return None;
    }
    if let Some((_, positions)) = CALLBACKS.iter().find(|(name, _)| *name == method) {
        return Some(positions);
    }
    if callee == "Array.from" {
        return Some(&[1]);
    }
    if matches!(method, "run" | "exit") {
        return als_file(&site.file, facts).then_some(if method == "run" { &[1] } else { &[0] });
    }
    (GLOBAL_OBJECTS.contains(&receiver) || METHODS.contains(&method)).then_some(&[])
}

fn als_file(file: &Path, facts: &dyn TsFactLookup) -> bool {
    facts.get_ts_facts(file).is_some_and(|facts| {
        facts.imported_bindings.iter().any(|binding| {
            binding.imported == "AsyncLocalStorage"
                && matches!(
                    binding.specifier.as_str(),
                    "async_hooks" | "node:async_hooks"
                )
        })
    })
}

/// Classify an unresolved call as a non-dispatching built-in; `None` when it is not one.
/// Callback arguments of callback-invoking built-ins are returned, never dropped.
pub(super) fn classify<'a>(
    site: &ResolvedCallSite,
    node_sites: &[&'a ResolvedCallSite],
    facts: &dyn TsFactLookup,
) -> Option<Callbacks<'a>> {
    let positions = positions(site, facts)?;
    let mut out = Callbacks::default();
    let call = facts.get_ts_facts(&site.file).and_then(|file| {
        file.value_flow
            .calls
            .iter()
            .find(|call| call.offset == site.offset)
    });
    for position in positions {
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
            ValueExpression::Callable(id) => {
                out.executed.extend(callback(&|other| other.offset == id.0));
            }
            ValueExpression::Named { name, .. } => {
                match callback(&|other| other.source_callee == *name) {
                    Some(found) if !found.has_unknown_target() => out.executed.push(found),
                    _ => out.unresolved.push(name.clone()),
                }
            }
            ValueExpression::Member(..) | ValueExpression::Call(..) => {
                out.unresolved.push(format!("argument {}", position + 1));
            }
            _ => {}
        }
    }
    Some(out)
}

/// Whether this candidate is a synchronous Node built-in module export.
pub(super) fn node_module(candidate: &KnownCallTargetRef<'_>) -> bool {
    matches!(candidate, KnownCallTargetRef::ModuleExport { specifier, repository_target: None, .. }
        if NODE_MODULES.contains(specifier))
}
