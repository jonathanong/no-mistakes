use crate::playwright::selectors::extract_playwright_selectors;
use crate::playwright::selectors::PlaywrightSelector;
use crate::playwright::selectors::SelectorMatcher;

fn attrs() -> Vec<String> {
    vec!["data-pw".to_string(), "data-testid".to_string()]
}

fn test_id_attrs() -> Vec<String> {
    vec!["data-testid".to_string()]
}

#[test]
fn extracts_playwright_selectors_simple() {
    let source = r#"
        await page.locator('[data-pw="open"]').click();
        await page.locator('[data-testid="publish"]').click();
    "#;
    let selectors = extract_playwright_selectors(source, &attrs(), &test_id_attrs());

    // Check that we extract specifically what was requested
    assert_eq!(selectors.len(), 2);
    assert!(selectors.contains(&PlaywrightSelector {
        attribute: "data-pw".to_string(),
        selector: r#"[data-pw="open"]"#.to_string(),
        matcher: SelectorMatcher::Exact("open".to_string()),
    }));
    assert!(selectors.contains(&PlaywrightSelector {
        attribute: "data-testid".to_string(),
        selector: r#"[data-testid="publish"]"#.to_string(),
        matcher: SelectorMatcher::Exact("publish".to_string()),
    }));
}

#[test]
fn extracts_playwright_selectors_with_test_id() {
    let source = r#"
        await page.getByTestId('submit-btn');
        await page.locator('button').getByTestId('cancel-btn');
    "#;
    let selectors = extract_playwright_selectors(source, &attrs(), &test_id_attrs());

    assert_eq!(selectors.len(), 2);
    assert!(selectors.contains(&PlaywrightSelector {
        attribute: "data-testid".to_string(),
        selector: "getByTestId(submit-btn)".to_string(),
        matcher: SelectorMatcher::Exact("submit-btn".to_string()),
    }));
    assert!(selectors.contains(&PlaywrightSelector {
        attribute: "data-testid".to_string(),
        selector: "getByTestId(cancel-btn)".to_string(),
        matcher: SelectorMatcher::Exact("cancel-btn".to_string()),
    }));
}

#[test]
fn extracts_playwright_selectors_ignores_unrelated() {
    let source = r#"
        await page.locator('.my-class').click();
        await page.locator('#my-id').click();
        const testId = "dynamic-" + id;
        await page.getByTestId(testId);
    "#;
    let selectors = extract_playwright_selectors(source, &attrs(), &test_id_attrs());

    // It should ignore pure class/id locators when they don't use the specific configured attributes
    assert_eq!(selectors.len(), 0);
}

#[test]
fn extracts_playwright_selectors_collects_duplicate_occurrences() {
    let source = r#"
        await page.locator('[data-pw="open"]').click();
        await page.locator('[data-pw="open"]').click();
        await page.getByTestId('submit-btn');
        await page.getByTestId('submit-btn');
    "#;
    let selectors = extract_playwright_selectors(source, &attrs(), &test_id_attrs());

    assert_eq!(selectors.len(), 4);
}
