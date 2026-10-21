use crate::config::v2::{schema::RuleDef, Effort, NoMistakesConfig};
use anyhow::Result;

pub(super) fn evaluates(config: &NoMistakesConfig, rule: &RuleDef) -> Result<bool> {
    Ok(config.effective_rule_analysis(rule)?.effort != Effort::Low)
}

pub(super) fn low_requested(config: &NoMistakesConfig) -> Result<bool> {
    for rule in config.rule_applications("postgres-require-query-annotation") {
        if !evaluates(config, rule)? {
            return Ok(true);
        }
    }
    Ok(false)
}
