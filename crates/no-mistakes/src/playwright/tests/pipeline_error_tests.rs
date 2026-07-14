use crate::playwright::analysis::context::{
    DiscoveredTestFile, RouteIndex, SelectorIndex, TestAnalysisContext, TestProjectContext,
};
use crate::playwright::analysis::test_file::analyze_test_file;
use crate::playwright::playwright_tests::TestPolicy;
use crate::playwright::selectors;
use crate::playwright::test_support::fixture_path;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn analyze_test_file_returns_error_for_missing_file() {
    // Exercises the `?` error branch in analyze_test_file when the file doesn't exist.
    let root = fixture_path(&["nextjs-coverage", "covered"]);
    let test_file = DiscoveredTestFile {
        path: PathBuf::from("/nonexistent/test.spec.ts"),
        contexts: vec![TestProjectContext {
            base_url: None,
            test_id_attributes: vec!["data-testid".to_string()],
        }],
    };
    let route_index = RouteIndex::default();
    let selector_index = SelectorIndex::default();
    let selector_regexes = selectors::compile_selector_regexes(&[], &BTreeMap::new());
    let context = TestAnalysisContext {
        root: &root,
        route_index: &route_index,
        selector_index: &selector_index,
        navigation_helpers: &[],
        selector_wrappers: &[],
        selector_regexes: &selector_regexes,
        test_policy: TestPolicy::default(),
    };
    let err = analyze_test_file(&test_file, &context);
    assert!(err.is_err(), "expected error for non-existent test file");
}

#[test]
fn analyze_test_file_returns_error_for_parse_failure() {
    let root = fixture_path(&["react-traits-components", "bad-file"]);
    let test_file = DiscoveredTestFile {
        path: root.join("app/components/Broken.tsx"),
        contexts: vec![TestProjectContext {
            base_url: None,
            test_id_attributes: vec!["data-testid".to_string()],
        }],
    };
    let route_index = RouteIndex::default();
    let selector_index = SelectorIndex::default();
    let selector_regexes = selectors::compile_selector_regexes(&[], &BTreeMap::new());
    let context = TestAnalysisContext {
        root: &root,
        route_index: &route_index,
        selector_index: &selector_index,
        navigation_helpers: &[],
        selector_wrappers: &[],
        selector_regexes: &selector_regexes,
        test_policy: TestPolicy::default(),
    };

    let err = analyze_test_file(&test_file, &context)
        .err()
        .expect("expected parse failure");

    assert!(!err.to_string().is_empty());
}
