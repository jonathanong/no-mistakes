use crate::playwright::selectors::types::{PlaywrightSelector, SelectorMatcher};

#[test]
fn test_playwright_selector_for_test() {
    let matcher = SelectorMatcher::Exact("value".to_string());
    let selector = PlaywrightSelector::for_test("data-testid", "submit", matcher.clone());

    assert_eq!(selector.attribute, "data-testid");
    assert_eq!(selector.selector, "submit");
    assert_eq!(selector.matcher, matcher);
}
