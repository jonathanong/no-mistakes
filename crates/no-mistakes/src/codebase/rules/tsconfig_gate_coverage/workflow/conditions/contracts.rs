use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    typed_scalar_expression_contexts_available, StaticExpressionType,
};
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
    declaration_names_valid(contract.inputs.keys())
        && contract.inputs.values().all(|declaration| {
            declaration.input_type.is_some_and(|input_type| {
                declaration
                    .default
                    .as_ref()
                    .is_none_or(|default| default_matches_type(default, input_type))
            })
        })
}

pub(super) fn workflow_call_contract_valid(contract: &WorkflowCallContract) -> bool {
    input_contract_valid(contract)
        && declaration_names_valid(contract.secrets.keys())
        && declaration_names_valid(contract.outputs.keys())
}

fn declaration_names_valid<'a>(names: impl Iterator<Item = &'a String>) -> bool {
    let names: Vec<&String> = names.collect();
    unique_contract_names(names.iter().copied()) && names.iter().all(|name| valid_identifier(name))
}

pub(super) fn valid_identifier(name: &str) -> bool {
    let mut characters = name.bytes();
    matches!(characters.next(), Some(b'A'..=b'Z' | b'a'..=b'z' | b'_'))
        && characters.all(|character| {
            matches!(
                character,
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'
            )
        })
}

fn default_matches_type(value: &JsonScalar, input_type: WorkflowCallInputType) -> bool {
    const INPUT_DEFAULT_CONTEXTS: &[&str] = &["github", "inputs", "vars"];
    match (input_type, value) {
        (WorkflowCallInputType::Boolean, JsonScalar::Bool(_))
        | (WorkflowCallInputType::Number, JsonScalar::Number(_)) => true,
        (input_type, JsonScalar::Text(value)) => {
            let expected = match input_type {
                WorkflowCallInputType::Boolean => StaticExpressionType::Boolean,
                WorkflowCallInputType::Number => StaticExpressionType::Number,
                WorkflowCallInputType::String => StaticExpressionType::String,
            };
            typed_scalar_expression_contexts_available(value, INPUT_DEFAULT_CONTEXTS, expected)
        }
        _ => false,
    }
}
