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
