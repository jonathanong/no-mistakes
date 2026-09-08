use super::EffectCallFact;
use crate::codebase::dependencies::extract::{FunctionCall, InvocationKind};
use std::collections::{HashMap, HashSet};

pub(crate) type EffectNames = HashMap<String, Option<String>>;

/// Projects configured effect occurrences from the canonical call collection.
///
/// Effects intentionally remain spelling-based: a configured terminal member
/// name still matches `client.createSubscriber()`, and a shadowed binding is
/// still reportable as an effect occurrence. Resolution-sensitive graph users
/// must inspect `FunctionCall::target_identity` instead.
pub(crate) fn collect_effect_calls(
    calls: &[FunctionCall],
    names: &EffectNames,
) -> Vec<EffectCallFact> {
    let canonical_by_offset = calls
        .iter()
        .enumerate()
        .filter(|(_, call)| is_effect_invocation(call, names))
        .fold(HashMap::new(), |mut canonical, (index, call)| {
            canonical
                .entry(effect_occurrence_key(call))
                .and_modify(|current| {
                    if prefers_ownership_record(call, &calls[*current]) {
                        *current = index;
                    }
                })
                .or_insert(index);
            canonical
        });
    let canonical_indices = canonical_by_offset
        .values()
        .copied()
        .collect::<HashSet<_>>();
    calls
        .iter()
        .enumerate()
        .filter(|call| canonical_indices.contains(&call.0) && is_effect_invocation(call.1, names))
        .filter_map(|(_, call)| {
            let (callee, category) = effect_match(&call.callee, names)?;
            Some(EffectCallFact {
                line: call.line as usize,
                callee: callee.to_string(),
                category: category.clone(),
                caller: call.syntactic_caller.clone(),
            })
        })
        .collect()
}

fn effect_occurrence_key(call: &FunctionCall) -> (u32, &str, bool) {
    (
        call.offset,
        &call.callee,
        matches!(call.invocation, InvocationKind::Construct),
    )
}

fn is_effect_invocation(call: &FunctionCall, names: &EffectNames) -> bool {
    matches!(
        call.invocation,
        InvocationKind::Call | InvocationKind::Construct
    ) && effect_match(&call.callee, names).is_some()
}

fn prefers_ownership_record(candidate: &FunctionCall, current: &FunctionCall) -> bool {
    match (candidate.caller.as_deref(), current.caller.as_deref()) {
        // An exported initializer can be collected both at module scope and
        // within its owning callable. Keep the callable-owned record.
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (Some(candidate), Some(current)) => {
            // Nested traversal projections may emit the same AST occurrence
            // under more than one callable scope. The shallowest scope is the
            // canonical owner; retain input order when scopes are peers.
            callable_scope_depth(candidate) < callable_scope_depth(current)
        }
        (None, None) => false,
    }
}

fn callable_scope_depth(scope: &str) -> usize {
    scope.matches('/').count()
}

fn effect_match<'a>(
    callee: &'a str,
    names: &'a EffectNames,
) -> Option<(&'a str, &'a Option<String>)> {
    names
        .get_key_value(callee)
        .map(|(name, category)| (name.as_str(), category))
        .or_else(|| {
            callee
                .rsplit_once('.')
                .and_then(|(_, terminal)| names.get_key_value(terminal))
                .map(|(name, category)| (name.as_str(), category))
        })
}
