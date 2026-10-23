use crate::playwright::selectors::types::{
    AppSelector, AppSelectorValue, PlaywrightSelector, SelectorMatcher,
};
use std::path::PathBuf;

#[test]
fn test_playwright_selector_for_test() {
    let matcher = SelectorMatcher::Exact("value".to_string());
    let selector = PlaywrightSelector::for_test("data-testid", "submit", matcher.clone());

    assert_eq!(selector.attribute, "data-testid");
    assert_eq!(selector.selector, "submit");
    assert_eq!(selector.matcher, matcher);
}

#[test]
fn app_selector_equality_and_order_ignore_line_and_recovered() {
    let literal = AppSelector {
        file: PathBuf::from("web/app/page.tsx"),
        attribute: "data-testid".to_string(),
        value: AppSelectorValue::Exact("save".to_string()),
        line: 4,
        recovered: false,
    };
    let recovered = AppSelector {
        file: PathBuf::from("web/app/page.tsx"),
        attribute: "data-testid".to_string(),
        value: AppSelectorValue::Exact("save".to_string()),
        line: 9,
        recovered: true,
    };
    let later_file = AppSelector {
        file: PathBuf::from("web/app/other.tsx"),
        attribute: "data-testid".to_string(),
        value: AppSelectorValue::Exact("save".to_string()),
        line: 1,
        recovered: false,
    };
    assert_eq!(literal, recovered);
    assert!(later_file < literal);
    let mut ordered = [literal.clone(), recovered.clone(), later_file.clone()];
    ordered.sort();
    assert_eq!(ordered[0].file, later_file.file);
    assert_eq!(ordered[1], literal);
    assert_eq!(ordered[2], recovered);
}
