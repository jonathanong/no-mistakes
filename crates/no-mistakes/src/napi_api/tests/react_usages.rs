use super::*;
use serde_json::json;

fn usages_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/react-traits-usages/basic/fixture"),
    )
}

#[test]
fn react_usages_json_returns_report() {
    let root = usages_fixture_root();
    let options = json!({
        "root": root,
        "target": "app/components/button.tsx#Button",
        "include": "stories,props"
    })
    .to_string();
    let output = react_usages_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["callsites"].as_array().unwrap().len(), 5);
    assert!(value["stories"].as_array().is_some());
    // tests section was not requested via `include`.
    assert!(value["tests"].is_null());
    assert_eq!(value["propTypes"][0], "ButtonProps");
}

#[test]
fn react_usages_json_requires_target() {
    let root = usages_fixture_root();
    let error = react_usages_json_impl(json!({ "root": root }).to_string()).unwrap_err();
    assert!(error.reason.contains("target is required"));
}

#[test]
fn react_usages_json_requires_target_without_root() {
    let error = react_usages_json_impl(json!({}).to_string()).unwrap_err();
    assert!(error.reason.contains("target is required for react usages"));
}
