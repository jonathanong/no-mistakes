use no_mistakes::codebase::rules::{RuleFinding, SuppressedFinding};
use std::time::Duration;

pub struct CheckTask<T> {
    pub findings: T,
    pub react_suppression_targets: Vec<Vec<no_mistakes::react_traits::ReactSuppressionTarget>>,
    pub pre_suppressed: Vec<SuppressedFinding>,
    pub suppression_sources: Vec<Option<String>>,
    pub advisories: Vec<RuleFinding>,
    pub analysis_warnings: Vec<String>,
    pub warning: Option<String>,
    pub duration: Duration,
}
