use crate::codebase::rules::tsconfig_gate_coverage::workflow::conditions::StaticBool;
use serde_yaml::Value;

#[derive(Default)]
pub(super) struct CheckoutState(bool);

impl CheckoutState {
    pub(super) fn available(&self) -> bool {
        self.0
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
                    !bindings.contains_key("repository")
                        && bindings
                            .get("path")
                            .is_none_or(|path| path.as_str() == Some("."))
                })
            });
    }
}
