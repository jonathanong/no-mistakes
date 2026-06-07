use crate::playwright::playwright_tests::{TestOccurrenceScope, TestStatus};
use crate::playwright::playwright_urls::api::{
    extract_playwright_url_literals_from_path, extract_playwright_url_literals_from_program,
    extract_playwright_url_literals_with_helpers, extract_playwright_url_occurrences,
    extract_playwright_urls,
};
use crate::playwright::playwright_urls::visitor::extract_playwright_url_occurrences_from_program;
use crate::playwright::test_support::fixture_source;
use std::path::Path;

#[test]
fn extract_playwright_url_literals_from_path_extracts_urls_successfully() {
    let src = "await page.goto('/dashboard'); navigateTo('/profile');";
    let urls = extract_playwright_url_literals_from_path(
        Path::new("fixture.ts"),
        src,
        &["navigateTo".to_string()],
    )
    .unwrap();
    assert_eq!(urls, vec!["/dashboard", "/profile"]);
}

#[test]
fn extract_playwright_url_literals_from_path_returns_err_on_parse_failure() {
    let src = "await page.goto('/dashboard'; // syntax error";
    let result = extract_playwright_url_literals_from_path(Path::new("fixture.ts"), src, &[]);
    assert!(result.is_err());
}

#[test]
fn callee_checks_handle_non_member_expressions() {
    let src = "goto('/')";
    let urls = extract_playwright_urls(src);
    assert!(urls.is_empty());
}

#[test]
fn extracts_page_goto_url() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "page-goto.ts"]);
    let urls = extract_playwright_urls(&src);
    assert_eq!(urls, vec!["/users/42"]);
}

#[test]
fn classifies_after_hook_urls_as_teardown_hooks() {
    let src = r#"
        test.afterEach({ timeout: 1000 }, async ({ page }) => {
            await page.goto("/cleanup");
        });
    "#;
    let urls =
        crate::playwright::ast::with_program(Path::new("fixture.ts"), src, |program, source| {
            extract_playwright_url_occurrences_from_program(program, source, &[])
        })
        .expect("fixture parses");

    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0].scope, TestOccurrenceScope::TeardownHook);
}

#[test]
fn dynamic_title_test_urls_are_scoped_to_tests() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "dynamic-title-scope.ts"]);
    let urls =
        crate::playwright::ast::with_program(Path::new("fixture.ts"), &src, |program, source| {
            extract_playwright_url_occurrences_from_program(program, source, &[])
        })
        .expect("fixture parses");

    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0].scope, TestOccurrenceScope::Test);
    assert_eq!(urls[0].test_name, None);
}

#[test]
fn extracts_click_href_selector() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "click-href.ts"]);
    let urls = extract_playwright_urls(&src);
    assert_eq!(urls, vec!["/dashboard"]);
}

#[test]
fn extracts_click_href_from_static_helper_and_ignores_empty_clicks() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "click-helper.ts"]);
    let urls = extract_playwright_urls(&src);
    assert_eq!(urls, vec!["/helper-click"]);
}

#[test]
fn extracts_double_quoted_goto_and_backtick_single_quoted_href() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "quoted-goto-click.ts"]);
    let urls = extract_playwright_urls(&src);
    assert_eq!(urls, vec!["/double", "/single"]);
}

#[test]
fn deduplicates_urls() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "duplicate-goto.ts"]);
    let urls = extract_playwright_urls(&src);
    assert_eq!(urls, vec!["/users/1"]);
}

#[test]
fn ignores_external_urls() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "external-urls.ts"]);
    let urls = extract_playwright_urls(&src);
    assert!(urls.is_empty());
}

#[test]
fn extracts_playwright_url_literals_from_program_test() {
    let src = r#"
        await page.goto("/c");
        await page.goto("/a");
        await page.goto("/b");
        await page.goto("/a"); // duplicate
        await navigateTo("/d");
    "#;
    let urls =
        crate::playwright::ast::with_program(Path::new("fixture.ts"), src, |program, source| {
            extract_playwright_url_literals_from_program(
                program,
                source,
                &["navigateTo".to_string()],
            )
        })
        .expect("fixture parses");

    assert_eq!(urls, vec!["/a", "/b", "/c", "/d"]);
}

#[test]
fn ignores_non_href_selectors() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "non-href-click.ts"]);
    let urls = extract_playwright_urls(&src);
    assert!(urls.is_empty());
}

#[test]
fn ignores_non_url_href_selector() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "non-url-href-click.ts"]);
    let urls = extract_playwright_urls(&src);
    assert!(urls.is_empty());
}

#[test]
fn empty_file_returns_empty() {
    let urls = extract_playwright_urls("");
    assert!(urls.is_empty());
}

#[test]
fn extracts_configured_navigation_helper_urls() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "navigation-helpers.ts"]);
    let urls = extract_playwright_url_literals_with_helpers(
        &src,
        &["navigateTo".to_string(), "testHelpers.openPath".to_string()],
    );
    assert_eq!(urls, vec!["/profile", "/settings", "/team"]);
}

#[test]
fn helper_url_extraction_skips_non_url_literals() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "helper-nested-url.ts"]);
    let urls = extract_playwright_url_literals_with_helpers(&src, &["navigateTo".to_string()]);
    assert_eq!(urls, vec!["/dynamic"]);
}

#[test]
fn navigation_helpers_use_only_the_target_argument() {
    let urls = extract_playwright_url_literals_with_helpers(
        "navigateTo('/orders', { redirect: '/login' });",
        &["navigateTo".to_string()],
    );
    assert_eq!(urls, vec!["/orders"]);
}

#[test]
fn extracts_to_have_url_assertion_paths() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "to-have-url.ts"]);
    let urls = extract_playwright_urls(&src);
    assert_eq!(
        urls,
        vec!["/settings", "/user/${username}/rss-feed-items/viewed"]
    );
}

#[test]
fn extracts_wait_for_url_page_url_match_and_static_route_helpers() {
    let urls = extract_playwright_urls(
        r#"
        const routes = {
            details: () => "/orders/42",
            overview: () => '/orders',
            metrics: () => `/orders/metrics`,
            dynamic: (id) => `/orders/${id}`,
        };
        // ghost: () => "/comment-only"
        const ignoredText = "text: () => '/string-only'";
        const account = { path: () => "/account" };
        const settings = { path: () => "/settings" };
        const analytics = { details() { return "/analytics"; } };
        await page.waitForURL(details());
        await page.waitForURL(routes.details());
        await page.waitForURL(analytics.details());
        await page.waitForURL("**/orders/globbed");
        await expect(page.url()).toMatch(overview());
        await expect.soft(page.url()).toMatch(/\/orders\/soft$/);
        await expect(page.url()).toMatch(metrics());
        await expect(page.url()).toMatch(dynamic("42"));
        await page.waitForURL(account.path());
        await page.waitForURL(settings.path());
        await frame.waitForURL(/^https:\/\/example.com\/orders\/absolute$/);
        await page.waitForURL(path());
        await page.waitForURL(getPath()());
        await page.waitForURL(/not-a-path/);
        await app.waitForURL("/unrelated");
        await page.goto();
        await page.goto(routeName);
        await page.waitForURL(ghost());
        await page.waitForURL(text());
        "#,
    );
    assert_eq!(
        urls,
        vec![
            "/orders",
            "/orders/42",
            "/orders/globbed",
            "/orders/metrics",
            "/orders/soft",
        ]
    );
}

#[test]
fn to_have_url_uses_first_url_literal_argument() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "to-have-url-label.ts"]);
    let urls = extract_playwright_url_literals_with_helpers(&src, &[]);
    assert_eq!(urls, vec!["/settings"]);
}

#[test]
fn extracts_occurrences_with_status_and_deduplicates() {
    let src = r#"
        test('active test', async ({ page }) => {
            await page.goto('/first');
            await page.goto('/second');
            await page.goto('/first'); // duplicate active
        });
        test.skip('skipped test', async ({ page }) => {
            await page.goto('/first'); // duplicate skipped
            await page.goto('/third');
        });
    "#;
    let occurrences = extract_playwright_url_occurrences(src);
    assert_eq!(
        occurrences,
        vec![
            ("/first".to_string(), TestStatus::Active),
            ("/first".to_string(), TestStatus::Skipped),
            ("/second".to_string(), TestStatus::Active),
            ("/third".to_string(), TestStatus::Skipped),
        ]
    );
}

#[test]
fn parenthesized_callee_is_supported() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "parenthesized-callee.ts"]);
    let urls = extract_playwright_urls(&src);
    assert_eq!(urls, vec!["/settings"]);
}

#[test]
fn bare_builtin_callees_are_ignored() {
    let src = fixture_source(&["ast-snippets", "playwright-urls", "bare-callees.ts"]);
    let urls = extract_playwright_urls(&src);
    assert!(urls.is_empty());
}

#[test]
fn folds_string_concatenation_into_interpolated_path() {
    // `'/users/' + id` becomes `/users/${id}`, which route matching treats as a dynamic
    // segment (#391). Nested `+`, parentheses, and template operands all fold.
    let goto = extract_playwright_urls("await page.goto('/users/' + id);");
    assert_eq!(goto, vec!["/users/${id}"]);

    let nested = extract_playwright_urls("await page.goto('/users/' + (id) + `/posts/${postId}`);");
    assert_eq!(nested, vec!["/users/${id}/posts/${postId}"]);

    let helper = extract_playwright_url_literals_with_helpers(
        "await navigateTo(page, '/user/' + targetUsername);",
        &["navigateTo".to_string()],
    );
    assert_eq!(helper, vec!["/user/${targetUsername}"]);
}

#[test]
fn static_only_concatenation_folds_to_concrete_path() {
    let urls = extract_playwright_urls("await page.goto('/users/' + '42');");
    assert_eq!(urls, vec!["/users/42"]);
}

#[test]
fn non_path_binary_expressions_extract_nothing_in_goto_path() {
    // The direct `goto` path folds the `+` chain; a non-candidate fold or a non-`+` binary
    // yields no URL (and never did — no regression).
    assert!(extract_playwright_urls("await page.goto(host + '/api/v1/users');").is_empty());
    assert!(extract_playwright_urls("await page.goto(offset * count);").is_empty());
}

#[test]
fn navigation_helper_binary_falls_back_to_default_extraction() {
    // The navigation-helper (visitor) path falls back to the default walk when the fold is
    // not a candidate path, so a nested URL literal is still extracted.
    let fallback = extract_playwright_url_literals_with_helpers(
        "await navigateTo(page, host + '/api/v1/users');",
        &["navigateTo".to_string()],
    );
    assert_eq!(fallback, vec!["/api/v1/users"]);

    // A non-`+` binary expression folds to nothing and extracts no URL.
    let arithmetic = extract_playwright_url_literals_with_helpers(
        "await navigateTo(page, offset * count);",
        &["navigateTo".to_string()],
    );
    assert!(arithmetic.is_empty());
}
