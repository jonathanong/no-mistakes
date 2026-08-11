use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::StaticBool;
use serde_yaml::Value;
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct CheckoutState(bool);

impl CheckoutState {
    pub(super) fn available(&self) -> bool {
        self.0
    }

    pub(super) fn local_action_available(
        &self,
        local_actions: &BTreeSet<String>,
        directory: &str,
    ) -> bool {
        self.available() && local_actions.contains(directory)
    }

    pub(super) fn observe(&mut self, step: &Value, condition: StaticBool) {
        self.0 |= condition == StaticBool::True
            && step
                .get("uses")
                .and_then(Value::as_str)
                .and_then(|target| target.strip_prefix("actions/checkout@"))
                .is_some_and(|reference| !reference.is_empty())
            && step.get("with").is_none_or(|bindings| {
                bindings.as_mapping().is_some_and(|bindings| {
                    !bindings.contains_key("repository") && !bindings.contains_key("path")
                })
            });
    }
}
