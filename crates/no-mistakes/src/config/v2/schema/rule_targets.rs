use super::RuleDef;

pub(super) fn rule_has_effective_test_target(rule: &RuleDef) -> bool {
    (rule_supports_vitest_test_targets(&rule.rule) && !rule.tests.vitest.is_empty())
        || (rule_supports_playwright_test_targets(&rule.rule) && !rule.tests.playwright.is_empty())
}

fn rule_supports_vitest_test_targets(rule_id: &str) -> bool {
    matches!(
        rule_id,
        "integration-test-no-mocks" | "test-no-unmocked-dynamic-imports"
    )
}

fn rule_supports_playwright_test_targets(rule_id: &str) -> bool {
    matches!(
        rule_id,
        "integration-test-no-mocks"
            | "test-no-unmocked-dynamic-imports"
            | "playwright-coverage"
            | "playwright-unique-test-ids"
            | "playwright-unique-html-ids"
            | "playwright-prefer-test-id-locators"
    )
}

/// Unbound Playwright rules still apply when `tests.playwright.apps` binds
/// each Playwright project to a frontend app.
pub(super) fn rule_has_playwright_apps_target(
    rule: &RuleDef,
    config: &super::NoMistakesConfig,
) -> bool {
    rule_supports_playwright_test_targets(&rule.rule)
        && rule.tests.playwright.is_empty()
        && !config.tests.playwright.apps.is_empty()
}
