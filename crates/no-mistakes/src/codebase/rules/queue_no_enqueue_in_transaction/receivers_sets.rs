use crate::codebase::dependencies::extract::{ValueFlowFacts, ValueParameter, ValueType};
use crate::fx::{FxHashMap, FxHashSet};

pub(in super::super) fn structural_params(
    parameters: &[ValueParameter],
) -> FxHashSet<(usize, &str)> {
    parameters
        .iter()
        .filter(|parameter| matches!(parameter.type_reference, Some(ValueType::Object(_))))
        .map(|parameter| (parameter.scope, parameter.name.as_str()))
        .collect()
}

/// Bindings whose initializer must not be trusted inside a transaction callback.
/// A `let` callback is not a value binding, so a later whole-name write is the
/// reassignment signal already recorded on the value flow.
pub(in super::super) fn invalidated_locals<'a>(
    flow: &'a ValueFlowFacts,
    declared: &'a [(usize, String, u32)],
) -> FxHashSet<(usize, &'a str)> {
    let mut names: FxHashSet<(usize, &'a str)> = flow
        .bindings
        .iter()
        .filter(|binding| binding.invalidated_at.is_some())
        .map(|binding| (binding.scope, binding.name.as_str()))
        .collect();
    let declared_at: FxHashMap<(usize, &str), u32> = declared
        .iter()
        .map(|(scope, name, at)| ((*scope, name.as_str()), *at))
        .collect();
    for write in &flow.identifier_writes {
        if !write.members.is_empty() {
            continue;
        }
        let Some(at) = declared_at.get(&(write.scope, write.name.as_str())) else {
            continue;
        };
        if write.at > *at {
            names.insert((write.scope, write.name.as_str()));
        }
    }
    names
}

pub(in super::super) fn written_members(
    flow: &ValueFlowFacts,
) -> FxHashSet<(usize, &str, Option<&str>)> {
    flow.identifier_writes
        .iter()
        .filter(|write| write.members.len() == 1)
        .map(|write| (write.scope, write.name.as_str(), write.members[0].as_deref()))
        .collect()
}
