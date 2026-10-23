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
