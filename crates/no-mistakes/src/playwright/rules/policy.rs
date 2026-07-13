use super::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use crate::config::v2::NoMistakesConfig;
use crate::playwright::analysis::types::UniqueSelectorPolicy;

pub fn configured(config: &NoMistakesConfig) -> bool {
    config.rule_configured(PLAYWRIGHT_COVERAGE)
        || config.rule_configured(PLAYWRIGHT_UNIQUE_TEST_IDS)
        || config.rule_configured(PLAYWRIGHT_UNIQUE_HTML_IDS)
        || config.rule_configured(PLAYWRIGHT_PREFER_TEST_ID_LOCATORS)
}

pub(super) fn unique_policy(unique_test_ids: bool, unique_html_ids: bool) -> UniqueSelectorPolicy {
    UniqueSelectorPolicy {
        test_ids: unique_test_ids,
        html_ids: unique_html_ids,
        aggregate: false,
        configured_html_id_selector: false,
    }
}
