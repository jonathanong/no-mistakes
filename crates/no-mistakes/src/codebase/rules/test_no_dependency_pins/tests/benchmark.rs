use crate::benchmark_support::{
    analyze_project_json, check_json, prepare_dependency_pins, project_dependency_pins,
};
use serde_json::Value;
use std::path::PathBuf;

#[test]
fn saved_jsx_benchmark_adapters_match_public_checks_without_projection_reparses() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/dependency-pin-jsx");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();

    crate::ast::begin_parse_count(&root);
    let prepared = prepare_dependency_pins(&root).unwrap();
    // Repeated projection must consume retained ranges after the extraction ASTs are evicted.
    assert_eq!(project_dependency_pins(&prepared), 128);
    assert_eq!(project_dependency_pins(&prepared), 128);
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 32, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");

    let full: Value = serde_json::from_str(&check_json(&root).unwrap()).unwrap();
    assert_eq!(full["rules"].as_array().unwrap().len(), 128);
    assert_eq!(full["warnings"], serde_json::json!([]));
    assert_eq!(full["advisories"], serde_json::json!([]));
    let options = serde_json::json!({
        "root": root,
        "config": ".pin-only-no-mistakes.yml",
        "reports": [{"type": "check"}],
    });
    let control: Value =
        serde_json::from_str(&analyze_project_json(options.to_string()).unwrap()).unwrap();
    assert_eq!(full, control["reports"][0]["result"]);
}

#[test]
fn saved_jsx_benchmark_public_check_preserves_invalid_pattern_error() {
    let saved = super::unit_fixture("benchmark-invalid");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let error = check_json(fixture.path()).unwrap_err();
    assert!(
        format!("{error:#}").contains("invalid pattern"),
        "{error:#}"
    );
}

#[test]
fn saved_jsx_benchmark_preparation_respects_default_test_selection() {
    let fixture =
        crate::test_support::materialize_saved_fixture(&super::fixture_root().join("prepared-jsx"));
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);
    let prepared = prepare_dependency_pins(&root).unwrap();
    // The stage adapter uses default selection and projects unsuppressed findings.
    assert_eq!(project_dependency_pins(&prepared), 6);
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 4, "{counts:#?}");
    assert!(!counts.contains_key(&root.join("src/helper.tsx")));
    assert!(!counts.contains_key(&root.join("src/pins.test.jsx")));
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");

    let fixture = crate::test_support::materialize_saved_fixture(&super::unit_fixture("custom"));
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);
    let prepared = prepare_dependency_pins(&root).unwrap();
    assert_eq!(project_dependency_pins(&prepared), 0);
    assert!(crate::ast::finish_parse_count(&root).is_empty());
}
