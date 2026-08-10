use crate::codebase::workflow_topology::model::{
    JsonScalar, WorkflowCallContract, WorkflowCallInputType,
};
use std::collections::BTreeSet;

pub(super) fn normalized_name(name: &str) -> String {
    name.to_lowercase()
}

pub(super) fn unique_contract_names<'a>(mut names: impl Iterator<Item = &'a String>) -> bool {
    let mut normalized = BTreeSet::new();
    names.all(|name| normalized.insert(normalized_name(name)))
}

pub(super) fn input_contract_valid(contract: &WorkflowCallContract) -> bool {
    unique_contract_names(contract.inputs.keys())
        && contract.inputs.values().all(|declaration| {
            declaration.input_type.is_some_and(|input_type| {
                declaration
                    .default
                    .as_ref()
                    .is_none_or(|default| default_matches_type(default, input_type))
            })
        })
}

fn default_matches_type(value: &JsonScalar, input_type: WorkflowCallInputType) -> bool {
    matches!(
        (input_type, value),
        (WorkflowCallInputType::Boolean, JsonScalar::Bool(_))
            | (WorkflowCallInputType::Number, JsonScalar::Number(_))
            | (WorkflowCallInputType::String, JsonScalar::Text(_))
    )
}
