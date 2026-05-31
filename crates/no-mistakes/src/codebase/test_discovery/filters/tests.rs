use super::*;

#[test]
fn vitest_fallback_rejects_tests_e2e_segment_pair() {
    assert!(!fallback_runner_match(
        TestRunner::Vitest,
        "apps/web/tests/e2e/home.test.ts"
    ));
}

#[test]
fn path_segment_pair_handles_empty_paths() {
    assert!(!has_path_segment_pair("", "tests", "e2e"));
}
