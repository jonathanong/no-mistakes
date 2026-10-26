use no_mistakes::codebase::rules::{RuleFinding, SuppressedFinding};
use std::time::Duration;

pub(crate) struct CheckTask<T> {
    pub(crate) findings: T,
    pub(crate) react_suppression_targets:
        Vec<Vec<no_mistakes::react_traits::ReactSuppressionTarget>>,
    pub(crate) pre_suppressed: Vec<SuppressedFinding>,
    pub(crate) suppression_sources: Vec<Option<String>>,
    pub(crate) advisories: Vec<RuleFinding>,
    pub(crate) warning: Option<String>,
    pub(crate) duration: Duration,
}
