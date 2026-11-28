use std::path::PathBuf;

use serde_json::json;

use super::super::playwright_check_json_impl;

#[test]
fn playwright_check_napi_parses_each_source_file_once() {
    let source = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser-count/playwright"),
    );
    let fixture = no_mistakes::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();

    no_mistakes::ast::begin_parse_count(&root);
    let output = playwright_check_json_impl(crate::options::test_json_arg(
        json!({ "root": root }).to_string(),
    ))
    .unwrap();
    let counts = no_mistakes::ast::finish_parse_count(&root);
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = [
        root.join("app/Widget.tsx"),
        root.join("app/page.tsx"),
        root.join("playwright.config.ts"),
        root.join("playwright.helper.ts"),
        root.join("tests/home.spec.ts"),
    ];

    assert_eq!(report["summary"]["totalRoutes"], 1);
    assert_eq!(counts.len(), expected.len(), "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    for file in expected {
        assert_eq!(counts.get(&file), Some(&1), "{counts:?}");
    }
}
