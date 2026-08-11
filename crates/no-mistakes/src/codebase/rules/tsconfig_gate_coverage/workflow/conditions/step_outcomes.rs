use super::StaticValue;
use serde_yaml::Value;
use std::collections::BTreeMap;

/// Outcomes that GitHub makes available through `steps.<id>.outcome` while a
/// job executes. Only outcomes proven by static execution are recorded.
#[derive(Clone, Default)]
pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) struct StepOutcomes(
    BTreeMap<String, StaticValue>,
);

impl StepOutcomes {
    pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn record(
        &mut self,
        step: &Value,
        outcome: StaticValue,
    ) {
        let Some(id) = step.get("id").and_then(Value::as_str) else {
            return;
        };
        self.0.insert(id.to_lowercase(), outcome);
    }

    pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn value(
        &self,
        id: &str,
    ) -> StaticValue {
        self.0
            .get(&id.to_lowercase())
            .cloned()
            .unwrap_or(StaticValue::Unknown)
    }
}
