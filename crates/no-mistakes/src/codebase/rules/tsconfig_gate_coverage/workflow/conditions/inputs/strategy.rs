use super::{InputState, StaticValue, STRATEGY_VALUE_PREFIX};
use serde_yaml::Value;

pub(crate) fn with_static_position_values(
    parent: &InputState,
    job: &Value,
    job_index: usize,
    job_total: usize,
) -> InputState {
    let mut inputs = parent.clone();
    if !has_matrix(job) {
        return inputs;
    }
    inputs.insert(
        format!("{STRATEGY_VALUE_PREFIX}job-index"),
        StaticValue::Number(job_index.to_string()),
    );
    inputs.insert(
        format!("{STRATEGY_VALUE_PREFIX}job-total"),
        StaticValue::Number(job_total.to_string()),
    );
    inputs
}

pub(crate) fn with_configuration_values(
    parent: &InputState,
    job: &Value,
    fail_fast: StaticValue,
    max_parallel: StaticValue,
) -> InputState {
    let mut inputs = parent.clone();
    if !has_matrix(job) {
        return inputs;
    }
    inputs.insert(format!("{STRATEGY_VALUE_PREFIX}fail-fast"), fail_fast);
    inputs.insert(format!("{STRATEGY_VALUE_PREFIX}max-parallel"), max_parallel);
    inputs
}

fn has_matrix(job: &Value) -> bool {
    job.get("strategy")
        .and_then(|strategy| strategy.get("matrix"))
        .is_some()
}
