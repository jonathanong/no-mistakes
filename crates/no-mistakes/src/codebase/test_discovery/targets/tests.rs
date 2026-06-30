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

#[test]
fn dotnet_target_without_project_path_runs_project_scope() {
    let target = target_for(
        TestRunner::Dotnet,
        None,
        Some("Company.App.Tests"),
        "dotnet-clients/tests/App.Tests/FeedServiceTests.cs",
    );

    assert_eq!(target.base_command, vec!["dotnet", "test"]);
    assert_eq!(target.runner_args, vec!["--no-restore"]);
    assert_eq!(
        test_file_arg(
            TestRunner::Dotnet,
            "dotnet-clients/tests/App.Tests/FeedServiceTests.cs"
        ),
        "dotnet-clients/tests/App.Tests/FeedServiceTests.cs"
    );
}

#[test]
fn swift_target_uses_file_parent_as_filter_without_project() {
    let target = target_for(
        TestRunner::Swift,
        Some("swift-clients/core"),
        None,
        "swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift",
    );

    assert_eq!(target.base_command, vec!["swift", "test"]);
    assert_eq!(
        target.runner_args,
        vec![
            "--package-path",
            "swift-clients/core",
            "--filter",
            "VouchaCoreTests"
        ]
    );
}

#[test]
fn swift_test_file_arg_keeps_literal_path_for_private_fallback() {
    assert_eq!(
        test_file_arg(TestRunner::Swift, "Tests/AppTests.swift"),
        "Tests/AppTests.swift"
    );
}
