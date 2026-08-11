use super::super::contracts::{normalized_name, workflow_call_contract_valid};
use super::bindings::normalized_bindings;
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    complete_expression_contexts_available, REUSABLE_CALL_SECRET_BINDING_CONTEXTS,
};
use crate::codebase::workflow_topology::model::WorkflowCallContract;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Secrets known to have crossed the current reusable-workflow boundary.
///
/// A directly triggered workflow has all repository secrets available. Explicit
/// bindings supply only their destination names; `secrets: inherit` preserves
/// the caller's availability state.
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SecretState {
    names: BTreeSet<String>,
    all: bool,
}

impl SecretState {
    pub(crate) fn direct() -> Self {
        Self {
            names: BTreeSet::new(),
            all: true,
        }
    }

    fn reusable(names: BTreeSet<String>, all: bool) -> Self {
        Self { names, all }
    }
}

pub(crate) fn callee_secrets(
    contract: &WorkflowCallContract,
    call_job: &Value,
    parent: &SecretState,
) -> Option<SecretState> {
    if !workflow_call_contract_valid(contract) {
        return None;
    }
    let (names, all) = if call_job.get("secrets").and_then(Value::as_str) == Some("inherit") {
        (parent.names.clone(), parent.all)
    } else {
        (explicit_secret_bindings(contract, call_job)?, false)
    };
    contract
        .secrets
        .iter()
        .filter(|(_, declaration)| declaration.required)
        .all(|(name, _)| all || names.contains(&normalized_name(name)))
        .then(|| SecretState::reusable(names, all))
}

fn explicit_secret_bindings(
    contract: &WorkflowCallContract,
    call_job: &Value,
) -> Option<BTreeSet<String>> {
    let bindings = match call_job.get("secrets") {
        Some(Value::Mapping(mapping)) => normalized_bindings(mapping)?,
        Some(_) => return None,
        None => BTreeMap::new(),
    };
    let declared_names: BTreeSet<String> = contract
        .secrets
        .keys()
        .map(|name| normalized_name(name))
        .collect();
    if bindings.keys().any(|name| !declared_names.contains(name))
        || bindings.values().any(|value| {
            !matches!(
                *value,
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
            )
        })
        || bindings.values().any(|value| {
            value.as_str().is_some_and(|value| {
                value.contains("${{")
                    && !complete_expression_contexts_available(
                        value,
                        REUSABLE_CALL_SECRET_BINDING_CONTEXTS,
                    )
            })
        })
    {
        return None;
    }
    Some(bindings.into_keys().collect())
}

#[cfg(test)]
mod tests;
