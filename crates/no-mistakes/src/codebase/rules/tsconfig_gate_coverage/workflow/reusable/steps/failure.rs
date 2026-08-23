use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::{
    StaticBool, StaticValue, StepOutcomes,
};
use serde_yaml::Value;

pub(super) fn record_unavailable(
    step: &Value,
    condition: StaticBool,
    outcomes: &mut StepOutcomes,
    failed: &mut bool,
    indeterminate: &mut bool,
) {
    if condition == StaticBool::True {
        outcomes.record(step, StaticValue::String("failure".to_string()));
        *failed = true;
    } else {
        *indeterminate = true;
    }
}
