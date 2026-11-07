use super::{binding_tag, value_call_at, Bindings};
use crate::codebase::dependencies::extract::{
    ValueCall, ValueExpression, ValueParameter, ValueProjection,
};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::postgres::TrustedSqlTag;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::Path;

#[path = "text_tag_arguments.rs"]
mod arguments;
#[path = "text_tag_callers.rs"]
mod callers;

#[derive(Clone, Hash, PartialEq, Eq)]
enum ProofKey {
    Call(u32, u32),
    Binding(usize, String, u32, Vec<String>),
    Parameter(usize, String, Vec<String>),
}

#[derive(Default)]
struct ProofState {
    active: FxHashSet<ProofKey>,
    done: FxHashMap<ProofKey, bool>,
    opaque_parameter_defaults: FxHashSet<usize>,
    callers: callers::State,
}

fn memoized(
    state: &mut ProofState,
    key: ProofKey,
    evaluate: impl FnOnce(&mut ProofState) -> bool,
) -> bool {
    if let Some(done) = state.done.get(&key) {
        return *done;
    }
    if !state.active.insert(key.clone()) {
        return false;
    }
    let proven = evaluate(state);
    state.active.remove(&key);
    state.done.insert(key, proven);
    proven
}

pub(crate) fn is_text_tag(
    bindings: &Bindings<'_>,
    file: &Path,
    call: &ValueCall,
    tags: &[TrustedSqlTag],
) -> bool {
    text_tag(bindings, file, call, tags, &mut ProofState::default())
}

pub(super) fn projection_span(
    bindings: &Bindings<'_>,
    file: &Path,
    value: &ValueExpression,
) -> Option<(u32, u32)> {
    let ValueExpression::Member(inner, property) = value else {
        return None;
    };
    if property != "text" {
        return None;
    }
    let ValueExpression::Call(start, end) = inner.as_ref() else {
        return None;
    };
    let call = value_call_at(bindings.facts, file, *start, *end)?;
    is_text_tag(bindings, file, call, bindings.trusted_tags).then_some((*start, *end))
}

fn text_tag(
    bindings: &Bindings<'_>,
    file: &Path,
    call: &ValueCall,
    tags: &[TrustedSqlTag],
    state: &mut ProofState,
) -> bool {
    let Some(ts) = bindings.facts.get_ts_facts(file) else {
        return false;
    };
    let Some(info) = ts.value_flow.tagged_templates.get(&(call.offset, call.end)) else {
        return false;
    };
    if !binding_tag::is_parameterized_tag(bindings, file, call, tags) {
        return false;
    }
    if binding_tag::is_bind_tag(bindings, file, call) {
        return true;
    }
    state
        .opaque_parameter_defaults
        .extend(info.opaque_parameter_defaults.iter().copied());
    memoized(state, ProofKey::Call(call.offset, call.end), |state| {
        call.arguments
            .iter()
            .enumerate()
            .skip(1)
            .all(|(index, argument)| {
                info.literal_arguments.contains(&index)
                    || arguments::bound_argument(bindings, file, argument, &[], tags, state)
            })
    })
}

#[cfg(test)]
#[path = "text_tag_callers_tests.rs"]
mod caller_tests;
#[cfg(test)]
#[path = "text_tag_tests.rs"]
mod tests;
