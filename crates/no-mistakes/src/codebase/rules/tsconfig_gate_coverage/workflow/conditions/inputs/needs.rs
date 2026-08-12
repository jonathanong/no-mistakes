use super::{InputState, StaticValue};
use serde_yaml::Value;
use std::collections::BTreeSet;

const NEEDS_RESULT_PREFIX: &str = "\0needs.";
const NEEDS_NOT_SKIPPED_SUFFIX: &str = ".not-skipped";
const NEEDS_OUTPUTS_SUFFIX: &str = ".outputs.";

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn inputs_with_needs_results(
    parent: &InputState,
    job: &Value,
    skipped: &BTreeSet<String>,
    failed: &BTreeSet<String>,
    executed: &BTreeSet<String>,
    outputs: &std::collections::BTreeMap<String, std::collections::BTreeMap<String, StaticValue>>,
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
        if executed.contains(&need) {
            for (name, value) in outputs.get(&need).into_iter().flatten() {
                inputs.insert(
                    format!("{NEEDS_RESULT_PREFIX}{need}{NEEDS_OUTPUTS_SUFFIX}{name}"),
                    value.clone(),
                );
            }
        }
    }
    inputs
}

pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn needs_output_value(
    job: &str,
    output: &str,
    inputs: &InputState,
) -> StaticValue {
    inputs
        .get(&format!(
            "{NEEDS_RESULT_PREFIX}{}{NEEDS_OUTPUTS_SUFFIX}{}",
            job.to_lowercase(),
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
