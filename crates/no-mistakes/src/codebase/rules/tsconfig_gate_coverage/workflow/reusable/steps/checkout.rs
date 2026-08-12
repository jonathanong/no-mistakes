use crate::codebase::rules::tsconfig_gate_coverage::{
    command_scan,
    workflow::conditions::{
        resolve_static_interpolations, EnvironmentState, InputState, StaticBool,
    },
};
use serde_yaml::Value;

#[derive(Default)]
pub(super) struct CheckoutState(bool);

impl CheckoutState {
    pub(super) fn available(&self) -> bool {
        self.0
    }

    pub(super) fn observe(
        &mut self,
        step: &Value,
        condition: StaticBool,
        inputs: &InputState,
        environment: &EnvironmentState,
    ) {
        self.0 |= condition == StaticBool::True
            && step
                .get("uses")
                .and_then(Value::as_str)
                .and_then(|target| target.strip_prefix("actions/checkout@"))
                .is_some_and(|reference| !reference.is_empty())
            && checkout_root_is_available(step.get("with"), inputs, environment);
    }
}

fn checkout_root_is_available(
    bindings: Option<&Value>,
    inputs: &InputState,
    environment: &EnvironmentState,
) -> bool {
    let Some(bindings) = bindings else {
        return true;
    };
    let Some(bindings) = bindings.as_mapping() else {
        return false;
    };
    !bindings.contains_key("repository")
        && !bindings.contains_key("sparse-checkout")
        && bindings.get("path").is_none_or(|path| {
            path.as_str()
                .and_then(|path| resolve_static_interpolations(path, inputs, environment))
                .is_some_and(|path| {
                    path.is_empty()
                        || command_scan::normalize_repo_relative(&path).as_deref() == Some(".")
                })
        })
}
