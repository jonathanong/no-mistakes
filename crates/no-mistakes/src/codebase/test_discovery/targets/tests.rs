use super::*;

#[test]
fn playwright_escapes_file_arg_as_regex_literal() {
    let target = target_for(
        TestRunner::Playwright,
        Some("playwright.config.ts"),
        Some("chromium"),
        "e2e/[locale].pw.ts",
    );

    assert_eq!(
        target.runner_args,
        vec![
            "--config",
            "playwright.config.ts",
            "--project",
            "chromium",
            "e2e/\\[locale\\]\\.pw\\.ts"
        ]
    );
}

#[test]
fn vitest_keeps_file_arg_literal() {
    let target = target_for(
        TestRunner::Vitest,
        Some("vitest.config.ts"),
        Some("unit"),
        "src/[locale].test.ts",
    );

    assert_eq!(
        target.runner_args,
        vec![
            "--config",
            "vitest.config.ts",
            "--project",
            "unit",
            "src/[locale].test.ts"
        ]
    );
}
