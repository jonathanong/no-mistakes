use super::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use crate::config::v2::{Effort, NoMistakesConfig, Unanalyzable};
use anyhow::Result;

pub(crate) const REWRITE: &str =
    "use a literal test id such as data-pw=\"save-button\", or a prop with a literal default";

pub(super) struct UniqueTestIdsPolicy {
    pub(super) low: bool,
    pub(super) unanalyzable: Unanalyzable,
}

/// Unique-test-ids checks each selector once. Any application that is not low
/// keeps recovery at max so a max application cannot lose findings.
pub(super) fn unique_test_ids_policy(config: &NoMistakesConfig) -> Result<UniqueTestIdsPolicy> {
    let mut saw = false;
    let mut low = true;
    let mut unanalyzable = Unanalyzable::Ignore;
    for rule in config
        .rules
        .iter()
        .filter(|rule| rule.enabled && rule.rule == PLAYWRIGHT_UNIQUE_TEST_IDS)
    {
        saw = true;
        let resolved = config.effective_rule_analysis(rule)?;
        if resolved.effort != Effort::Low {
            low = false;
        }
        unanalyzable = stricter(unanalyzable, resolved.unanalyzable);
    }
    if !saw || !low {
        return Ok(UniqueTestIdsPolicy {
            low: false,
            unanalyzable: Unanalyzable::Report,
        });
    }
    Ok(UniqueTestIdsPolicy {
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
mod tests {
    use super::*;
    use crate::config::v2::schema::{RuleDef, RuleScope};
    use crate::config::v2::EffectiveRuleAnalysis;

    fn rule(id: &str, effort: Effort, unanalyzable: Unanalyzable) -> RuleDef {
        RuleDef {
            rule: id.to_string(),
            scope: Some(RuleScope::Repository),
            resolved_analysis: Some(EffectiveRuleAnalysis {
                effort,
                unanalyzable,
            }),
            ..RuleDef::default()
        }
    }

    fn config(rules: Vec<RuleDef>) -> NoMistakesConfig {
        NoMistakesConfig {
            rules,
            ..NoMistakesConfig::default()
        }
    }

    #[test]
    fn unique_test_ids_stays_max_when_any_application_is_not_low() {
        let mixed = unique_test_ids_policy(&config(vec![
            rule(
                PLAYWRIGHT_UNIQUE_TEST_IDS,
                Effort::Low,
                Unanalyzable::Ignore,
            ),
            rule(
                PLAYWRIGHT_UNIQUE_TEST_IDS,
                Effort::Max,
                Unanalyzable::Advise,
            ),
        ]))
        .unwrap();
        assert!(!mixed.low);

        let low = unique_test_ids_policy(&config(vec![rule(
            PLAYWRIGHT_UNIQUE_TEST_IDS,
            Effort::Low,
            Unanalyzable::Advise,
        )]))
        .unwrap();
        assert!(low.low);
        assert_eq!(low.unanalyzable, Unanalyzable::Advise);
    }

    #[test]
    fn skip_dynamic_recovery_requires_every_selector_consumer_to_be_low() {
        let unique_low = config(vec![rule(
            PLAYWRIGHT_UNIQUE_TEST_IDS,
            Effort::Low,
            Unanalyzable::Report,
        )]);
        assert!(skip_dynamic_recovery(&unique_low, false).unwrap());
        assert!(!skip_dynamic_recovery(&unique_low, true).unwrap());

        let mixed = config(vec![
            rule(
                PLAYWRIGHT_UNIQUE_TEST_IDS,
                Effort::Low,
                Unanalyzable::Report,
            ),
            rule(PLAYWRIGHT_COVERAGE, Effort::Max, Unanalyzable::Report),
        ]);
        assert!(!skip_dynamic_recovery(&mixed, false).unwrap());
    }
}
