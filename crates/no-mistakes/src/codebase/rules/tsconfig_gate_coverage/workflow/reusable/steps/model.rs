use std::collections::BTreeSet;

pub(crate) struct StepScan {
    pub(crate) projects: BTreeSet<String>,
    pub(crate) failed: bool,
    pub(crate) indeterminate: bool,
}
