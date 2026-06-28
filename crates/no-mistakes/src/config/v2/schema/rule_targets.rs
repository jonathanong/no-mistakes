use super::RuleDef;

pub(super) fn rule_has_effective_test_target(rule: &RuleDef) -> bool {
    (rule_supports_vitest_test_targets(&rule.rule) && !rule.tests.vitest.is_empty())
        || (rule_supports_playwright_test_targets(&rule.rule) && !rule.tests.playwright.is_empty())
}

fn rule_supports_vitest_test_targets(rule_id: &str) -> bool {
    matches!(rule_id, "test-no-unmocked-dynamic-imports")
}

fn rule_supports_playwright_test_targets(rule_id: &str) -> bool {
    matches!(
        rule_id,
        "test-no-unmocked-dynamic-imports"
            | "playwright-coverage"
            | "playwright-unique-test-ids"
            | "playwright-unique-html-ids"
            | "playwright-prefer-test-id-locators"
    )
}
