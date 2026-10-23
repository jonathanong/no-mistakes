use crate::codebase::check_facts::CheckFactPlan;
use crate::config::v2::{Effort, NoMistakesConfig};
use anyhow::Result;

/// Declare both projections for mixed applications; low-only never builds max facts.
#[doc(hidden)]
pub fn configure_effort_plan(config: &NoMistakesConfig, plan: &mut CheckFactPlan) -> Result<()> {
    let mut low = false;
    let mut max = false;
    for rule in config.rule_applications(super::RULE_ID) {
        if config.effective_rule_analysis(rule)?.effort == Effort::Low {
            low = true;
        } else {
            max = true;
        }
    }
    plan.nextjs_caching = max;
    plan.nextjs_caching_low = low;
    Ok(())
}
