use super::helpers::{
    extract_playwright_selector_occurrences, extract_playwright_selectors,
    extract_playwright_selectors_with_regexes, extract_playwright_text_locator_occurrences,
    extract_playwright_text_locators,
};
use crate::playwright::playwright_tests::TestStatus;
use crate::playwright::selectors::compile_selector_regexes_with_html_ids;
use std::collections::BTreeMap;
use std::path::Path;

fn attrs() -> Vec<String> {
    vec!["data-testid".to_string(), "data-pw".to_string()]
}

#[test]
fn extracts_playwright_css_and_test_id_selectors() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "playwright-css-and-testid.ts",
    ]);
    let selectors = extract_playwright_selectors(&source, &attrs(), &["data-testid".to_string()]);
    assert!(selectors.iter().any(|s| s.selector == "getByTestId(save)"));
    assert!(selectors
        .iter()
        .any(|s| s.selector == "[data-testid^='user-']"));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-pw$="button"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-pw*="nav"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-pw="exact"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == "getByTestId(/^account-/)"));
}

#[test]
fn marks_selectors_inside_skipped_and_conditional_tests() {
    let selectors = extract_playwright_selector_occurrences(
        r#"
        test.skip('skipped', async ({ page }) => { await page.getByTestId('skipped'); });
        test.fixme('fixme test', async ({ page }) => { await page.getByTestId('fixme'); });
        if (process.env.E2E) {
            test('conditional wrapper', async ({ page }) => {
                await page.getByTestId('conditional-wrapper');
            });
        } else {
            test('conditional alternate', async ({ page }) => {
                await page.locator('[data-testid="conditional-alternate"]');
            });
        }
        featureFlag && test('logical wrapper', async ({ page }) => {
            await page.getByTestId('logical-wrapper');
        });
        featureFlag
            ? test('ternary consequent', async ({ page }) => {
                await page.getByTestId('ternary-consequent');
            })
            : test('ternary alternate', async ({ page }) => {
                await page.getByTestId('ternary-alternate');
            });
        test('active', async ({ page }) => {
            await page.getByTestId('active');
            await page.getByTestId('active');
        });
        test.skip(({ browserName }) => browserName === 'webkit', 'conditional');
        test('file scope annotation', async ({ page }) => {
            await page.getByTestId('scope-annotation');
        });
        "#,
        &attrs(),
        &["data-testid".to_string()],
    );

    assert_eq!(
        selectors,
        vec![
            (
                r#"[data-testid="conditional-alternate"]"#.to_string(),
                TestStatus::Conditional
            ),
            ("getByTestId(active)".to_string(), TestStatus::Active),
            (
                "getByTestId(conditional-wrapper)".to_string(),
                TestStatus::Conditional
            ),
            ("getByTestId(fixme)".to_string(), TestStatus::Skipped),
            (
                "getByTestId(logical-wrapper)".to_string(),
                TestStatus::Conditional
            ),
            (
                "getByTestId(scope-annotation)".to_string(),
                TestStatus::Conditional
            ),
            ("getByTestId(skipped)".to_string(), TestStatus::Skipped),
            (
                "getByTestId(ternary-alternate)".to_string(),
                TestStatus::Conditional
            ),
            (
                "getByTestId(ternary-consequent)".to_string(),
                TestStatus::Conditional
            ),
        ]
    );
}

#[test]
fn css_attribute_selectors_must_be_used_by_playwright_selector_calls() {
    let source = r#"
        const unused = '[data-testid="save"]';
        await page.locator('[data-testid="publish"]').click();
        await page.click(`[data-pw="open"]`);
        await page.type('[data-testid="search"]', 'query');
        await page.$eval('[data-pw="panel"]', node => node.textContent);
        await page.$$eval('[data-testid="items"]', nodes => nodes.length);
        await page.frameLocator('[data-pw="frame"]').locator('[data-testid="inside"]');
        await page.dragAndDrop('[data-testid="source"]', '[data-pw="target"]');
    "#;
    let selectors = extract_playwright_selectors(source, &attrs(), &["data-testid".to_string()]);
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-testid="publish"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-pw="open"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-testid="search"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-pw="panel"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-testid="items"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-pw="frame"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-testid="inside"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-testid="source"]"#));
    assert!(selectors
        .iter()
        .any(|s| s.selector == r#"[data-pw="target"]"#));
    assert!(selectors
        .iter()
        .all(|s| s.selector != r#"[data-testid="save"]"#));
}

#[test]
fn extracts_html_ids_playwright_selectors() {
    let regexes = compile_selector_regexes_with_html_ids(
        &["data-testid".to_string()],
        &BTreeMap::new(),
        true,
    );
    let playwright_selectors = extract_playwright_selectors_with_regexes(
        Path::new("tests/app.spec.ts"),
        r#"
        await page.locator('#save').click();
        await page.locator('button#user-42 .label').click();
        await page.locator('#save, #publish').click();
        await page.locator('[id="save"]').click();
        "#,
        &regexes,
        &["data-testid".to_string()],
    )
    .unwrap();
    let values: std::collections::BTreeSet<(String, String)> = playwright_selectors
        .iter()
        .map(|s| (s.attribute.clone(), s.selector.clone()))
        .collect();
    assert_eq!(
        values,
        std::collections::BTreeSet::from([
            ("id".to_string(), "#publish".to_string()),
            ("id".to_string(), "#save".to_string()),
            ("id".to_string(), "#user-42".to_string()),
            ("id".to_string(), r#"[id="save"]"#.to_string()),
        ])
    );
}

#[test]
fn extracts_playwright_text_locators() {
    let locators = extract_playwright_text_locators(
        r#"
        await page.getByRole("button", { name: "Discuss" }).click();
        await page.getByRole("button", { ...roleOptions, name: "Spread role" }).click();
        await page.getByRole(`button`, { exact: true, [`name`]: "Ignored", name: `Template name` });
        await page.getByRole("button", { name: "Hidden role", includeHidden: true });
        await page.getByRole("button", { "name": "String key role", "exact": true, "includeHidden": false });
        await page.getByRole("button", { name: "Bad exact", exact: "yes" });
        await page.getByRole("button", { name: "Bad hidden", includeHidden: includeHidden });
        await page.getByRole("button", { name: "Computed exact", exact: isExact });
        await page.getByRole("checkbox", { name: "Subscribe", checked: true });
        await page.getByRole("button", { name: "Described", description: "Primary" });
        await page.getByRole("button");
        await page.getByRole("button", "not options");
        await page.getByText("Welcome back").click();
        await page.getByText("Welcome back").click();
        await page.getByText(`Exact text`, { exact: true }).click();
        await page.getByText("Spread exact", { ...textOptions, exact: false }).click();
        await page.getByText("Method exact", { exact() { return true; } }).click();
        await page.getByText("Unknown exact", { exact: isExact }).click();
        await page.getByText("Loose text", "not options").click();
        await page.getByLabel(`Email`).fill("a@b.com");
        await page.getByLabel("Full name", { exact: false }).fill("Ada");
        await page.getByPlaceholder("Search").fill("x");
        await page.getByText(dynamic);
        await page.getByRole("button", { name: /Save/ });
        "#,
    );
    assert_eq!(
        locators,
        vec![
            (
                "role".to_string(),
                "Discuss".to_string(),
                Some("button".to_string())
            ),
            (
                "role".to_string(),
                "Hidden role".to_string(),
                Some("button".to_string())
            ),
            (
                "role".to_string(),
                "String key role".to_string(),
                Some("button".to_string())
            ),
            ("text".to_string(), "Exact text".to_string(), None),
            ("text".to_string(), "Welcome back".to_string(), None),
            ("label".to_string(), "Email".to_string(), None),
            ("label".to_string(), "Full name".to_string(), None),
            ("placeholder".to_string(), "Search".to_string(), None),
        ]
    );
}

#[test]
fn extracts_text_locator_status_and_ignores_unsupported_shapes() {
    let source = crate::playwright::test_support::fixture_source(&[
        "ast-snippets",
        "selectors",
        "playwright-text-locators-branches.ts",
    ]);
    let locators = extract_playwright_text_locator_occurrences(&source);

    assert_eq!(locators.len(), 8);
    assert!(locators.contains(&(
        "role".to_string(),
        "Save".to_string(),
        Some("button".to_string()),
        TestStatus::Active,
        Some("active role".to_string()),
        vec!["settings".to_string()]
    )));
    assert!(locators.contains(&(
        "text".to_string(),
        "Skip me".to_string(),
        None,
        TestStatus::Skipped,
        Some("skipped text".to_string()),
        vec![]
    )));
    assert!(locators.contains(&(
        "text".to_string(),
        "Annotation text".to_string(),
        None,
        TestStatus::Conditional,
        Some("annotation text".to_string()),
        vec![]
    )));
    assert!(locators.iter().any(|(_, text, _, status, _, _)| {
        text == "Conditional label" && *status == TestStatus::Conditional
    }));
    assert!(locators.iter().any(|(_, text, _, status, _, _)| {
        text == "Conditional placeholder" && *status == TestStatus::Conditional
    }));
    assert!(locators.iter().any(|(_, text, _, status, _, _)| {
        text == "Logical text" && *status == TestStatus::Conditional
    }));
    assert!(locators.iter().any(|(_, text, _, status, _, _)| {
        text.starts_with("Ternary ") && *status == TestStatus::Conditional
    }));
    assert!(!locators
        .iter()
        .any(|(_, text, _, _, _, _)| text == "Dynamic filter"));
}
