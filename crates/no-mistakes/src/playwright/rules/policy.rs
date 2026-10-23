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

pub(super) fn unique_policy(
    unique_test_ids: bool,
    unique_html_ids: bool,
    literal_test_ids_only: bool,
    literal_html_ids_only: bool,
) -> UniqueSelectorPolicy {
    UniqueSelectorPolicy {
        test_ids: unique_test_ids,
        html_ids: unique_html_ids,
        aggregate: false,
        configured_html_id_selector: false,
        literal_test_ids_only,
        // Another selection may use `id` as a configured test-ID attribute.
        // Only the HTML rule's own selection may narrow those duplicates.
        literal_html_ids_only: unique_html_ids && literal_html_ids_only,
    }
}

#[cfg(test)]
mod tests {
    use super::unique_policy;
    use crate::playwright::analysis::duplicates::build_duplicate_selectors;
    use crate::playwright::selectors::{AppSelector, AppSelectorValue};
    use std::path::{Path, PathBuf};

    #[test]
    fn html_low_does_not_narrow_a_selection_without_the_html_rule() {
        let selectors: Vec<_> = ["a", "b"]
            .into_iter()
            .map(|name| AppSelector {
                file: PathBuf::from(format!("/repo/{name}.tsx")),
                attribute: "id".to_string(),
                value: AppSelectorValue::Exact("same".to_string()),
                line: 1,
                recovered: true,
            })
            .collect();
        for (html_selected, expected) in [(false, 2), (true, 0)] {
            let mut policy = unique_policy(true, html_selected, false, true);
            policy.configured_html_id_selector = true;
            let duplicates = build_duplicate_selectors(Path::new("/repo"), &selectors, policy);
            assert_eq!(duplicates.len(), expected, "HTML selected: {html_selected}");
        }
    }
}
