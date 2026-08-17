use super::*;

#[test]
fn vitest_fallback_rejects_tests_e2e_segment_pair() {
    assert!(!fallback_runner_match(
        TestRunner::Vitest,
        "apps/web/tests/e2e/home.test.ts"
    ));
}

#[test]
fn playwright_fallback_requires_test_shaped_file() {
    assert!(!fallback_runner_match(
        TestRunner::Playwright,
        "tests/e2e/helpers.ts"
    ));
    assert!(fallback_runner_match(
        TestRunner::Playwright,
        "tests/e2e/home.spec.ts"
    ));
}

#[test]
fn path_segment_pair_handles_empty_paths() {
    assert!(!has_path_segment_pair("", "tests", "e2e"));
}

#[test]
fn language_fallback_matches_configured_test_shapes() {
    assert!(fallback_runner_match(TestRunner::Python, "app/test_users.py"));
    assert!(fallback_runner_match(TestRunner::Go, "pkg/ping_test.go"));
    assert!(fallback_runner_match(TestRunner::Cargo, "app/src/tests.rs"));
    assert!(fallback_runner_match(
        TestRunner::Cargo,
        "app/tests/integration.rs"
    ));
    assert!(fallback_runner_match(
        TestRunner::Rails,
        "spec/jobs/welcome_job_spec.rb"
    ));
    assert!(fallback_runner_match(
        TestRunner::Php,
        "tests/UserControllerTest.php"
    ));
    assert!(!fallback_runner_match(TestRunner::Python, "app/users.py"));
    assert!(!fallback_runner_match(TestRunner::Cargo, "app/src/lib.rs"));
}
