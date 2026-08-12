use super::{InputState, StaticValue};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

const NEEDS_RESULT_PREFIX: &str = "\0needs.";
const NEEDS_NOT_SKIPPED_SUFFIX: &str = ".not-skipped";

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn inputs_with_needs_results(
    parent: &InputState,
    job: &Value,
    skipped: &BTreeSet<String>,
    failed: &BTreeSet<String>,
    executed: &BTreeSet<String>,
    outputs: &BTreeMap<String, BTreeMap<String, StaticValue>>,
) -> InputState {
    let mut inputs = parent.clone();
    for need in crate::codebase::workflow_topology::value_primitives::string_list(job.get("needs"))
    {
        let need = need.to_lowercase();
        let result = if failed.contains(&need) {
            StaticValue::String("failure".to_string())
        } else if skipped.contains(&need) {
            StaticValue::String("skipped".to_string())
        } else if executed.contains(&need) {
            StaticValue::String("success".to_string())
        } else {
            StaticValue::Unknown
        };
        inputs.insert(format!("{NEEDS_RESULT_PREFIX}{need}.result"), result);
        if executed.contains(&need) {
            inputs.insert(
                format!("{NEEDS_RESULT_PREFIX}{need}{NEEDS_NOT_SKIPPED_SUFFIX}"),
                StaticValue::Bool(true),
            );
        }
        if let Some(outputs) = outputs.get(&need) {
            for (name, value) in outputs {
                inputs.insert(
                    format!(
                        "{NEEDS_RESULT_PREFIX}{need}.outputs.{}",
                        name.to_lowercase()
                    ),
                    value.clone(),
                );
            }
        }
    }
    inputs
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn needs_output_value(
    need: &str,
    output: &str,
    inputs: &InputState,
) -> StaticValue {
    inputs
        .get(&format!(
            "{NEEDS_RESULT_PREFIX}{}.outputs.{}",
            need.to_lowercase(),
            output.to_lowercase()
        ))
        .cloned()
        .unwrap_or(StaticValue::Unknown)
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn needs_result_not_skipped(
    name: &str,
    inputs: &InputState,
) -> bool {
    inputs.contains_key(&format!(
        "{NEEDS_RESULT_PREFIX}{}{NEEDS_NOT_SKIPPED_SUFFIX}",
        name.to_lowercase()
    ))
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn needs_result_value(
    name: &str,
    inputs: &InputState,
) -> StaticValue {
    inputs
        .get(&format!(
            "{NEEDS_RESULT_PREFIX}{}.result",
            name.to_lowercase()
        ))
        .cloned()
        .unwrap_or(StaticValue::Unknown)
}

#[cfg(test)]
#[path = "needs_tests.rs"]
mod tests;
