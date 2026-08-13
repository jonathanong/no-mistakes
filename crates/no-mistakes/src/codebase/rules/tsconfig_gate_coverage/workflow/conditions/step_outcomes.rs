use super::StaticValue;
use serde_yaml::Value;
use std::collections::BTreeMap;

/// Results GitHub exposes for completed steps. A tolerated failure has
/// `outcome: failure` but `conclusion: success`, so retain both independently.
#[derive(Clone, Default)]
pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) struct StepOutcomes {
    outcomes: BTreeMap<String, StaticValue>,
    conclusions: BTreeMap<String, StaticValue>,
}

impl StepOutcomes {
    pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn record(
        &mut self,
        step: &Value,
        result: StaticValue,
    ) {
        self.record_with_conclusion(step, result.clone(), result);
    }

    pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn record_with_conclusion(
        &mut self,
        step: &Value,
        outcome: StaticValue,
        conclusion: StaticValue,
    ) {
        let Some(id) = step.get("id").and_then(Value::as_str) else {
            return;
        };
        let id = id.to_lowercase();
        self.outcomes.insert(id.clone(), outcome);
        self.conclusions.insert(id, conclusion);
    }

    pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn value(
        &self,
        id: &str,
    ) -> StaticValue {
        self.outcomes
            .get(&id.to_lowercase())
            .cloned()
            .unwrap_or(StaticValue::Unknown)
    }

    pub(in crate::codebase::rules::tsconfig_gate_coverage::workflow) fn conclusion(
        &self,
        id: &str,
    ) -> StaticValue {
        self.conclusions
            .get(&id.to_lowercase())
            .cloned()
            .unwrap_or(StaticValue::Unknown)
    }
}

#[cfg(test)]
mod tests;
