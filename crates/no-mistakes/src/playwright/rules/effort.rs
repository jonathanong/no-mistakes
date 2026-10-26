use super::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use crate::config::v2::{Effort, NoMistakesConfig, Unanalyzable};
use anyhow::Result;

pub(crate) const REWRITE: &str =
    "use a literal test id such as data-pw=\"save-button\", or a prop with a literal default";
pub(crate) const HTML_ID_REWRITE: &str =
    "use a literal HTML id such as id=\"save-button\", or a prop with a literal default";

pub(super) struct RuleEffortPolicy {
    pub(super) low: bool,
    pub(super) unanalyzable: Unanalyzable,
}

/// Unique-test-ids checks each selector once. Any application that is not low
/// keeps recovery at max so a max application cannot lose findings.
pub(super) fn unique_test_ids_policy(config: &NoMistakesConfig) -> Result<RuleEffortPolicy> {
    rule_effort_policy(config, PLAYWRIGHT_UNIQUE_TEST_IDS)
}

/// Keep HTML-ID recovery at max when any application of this rule needs max.
pub(super) fn unique_html_ids_policy(config: &NoMistakesConfig) -> Result<RuleEffortPolicy> {
    rule_effort_policy(config, PLAYWRIGHT_UNIQUE_HTML_IDS)
}

/// Coverage checks each selector once. Any application that is not low keeps
/// recovery at max so a max application cannot lose findings.
pub(super) fn coverage_policy(config: &NoMistakesConfig) -> Result<RuleEffortPolicy> {
    rule_effort_policy(config, PLAYWRIGHT_COVERAGE)
}

fn rule_effort_policy(config: &NoMistakesConfig, rule_id: &str) -> Result<RuleEffortPolicy> {
    let mut saw = false;
    let mut low = true;
    let mut unanalyzable = Unanalyzable::Ignore;
    for rule in config
        .rules
        .iter()
        .filter(|rule| rule.enabled && rule.rule == rule_id)
    {
        saw = true;
        let resolved = config.effective_rule_analysis(rule)?;
        if resolved.effort != Effort::Low {
            low = false;
        }
        unanalyzable = stricter(unanalyzable, resolved.unanalyzable);
    }
    if !saw || !low {
        return Ok(RuleEffortPolicy {
            low: false,
            unanalyzable: Unanalyzable::Report,
        });
    }
    Ok(RuleEffortPolicy {
        low: true,
        unanalyzable,
    })
}

/// Skip identifier/helper/cross-file recovery only when every Playwright
/// selector consumer in this invocation is effectively low. Graph selector
/// demand keeps recovery.
pub(crate) fn skip_dynamic_recovery(
    config: &NoMistakesConfig,
    graph_selectors: bool,
) -> Result<bool> {
    if graph_selectors {
        return Ok(false);
    }
    let mut any = false;
    for rule_id in [
        PLAYWRIGHT_COVERAGE,
        PLAYWRIGHT_UNIQUE_TEST_IDS,
        PLAYWRIGHT_UNIQUE_HTML_IDS,
        PLAYWRIGHT_PREFER_TEST_ID_LOCATORS,
    ] {
        for rule in config.rule_applications(rule_id) {
            any = true;
            if config.effective_rule_analysis(rule)?.effort != Effort::Low {
                return Ok(false);
            }
        }
    }
    Ok(any)
}

fn stricter(left: Unanalyzable, right: Unanalyzable) -> Unanalyzable {
    match (left, right) {
        (Unanalyzable::Report, _) | (_, Unanalyzable::Report) => Unanalyzable::Report,
        (Unanalyzable::Advise, _) | (_, Unanalyzable::Advise) => Unanalyzable::Advise,
        _ => Unanalyzable::Ignore,
    }
}

#[cfg(test)]
mod tests;
