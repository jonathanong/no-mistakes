use serde_yaml::Value;
use std::collections::BTreeSet;

pub(super) fn run_command(step: &Value) -> Option<&str> {
    step.get("run").and_then(Value::as_str)
}

pub(crate) struct StepScan {
    pub(crate) projects: BTreeSet<String>,
    pub(crate) failed: bool,
    pub(crate) indeterminate: bool,
}

impl StepScan {
    pub(super) fn new(projects: BTreeSet<String>, failed: bool, indeterminate: bool) -> Self {
        Self {
            projects,
            failed,
            indeterminate,
        }
    }
}
