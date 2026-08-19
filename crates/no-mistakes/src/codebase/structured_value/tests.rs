use super::{is_jsonc_path, parse_structured_value};
use serde_yaml::Value;
use std::path::Path;

#[test]
fn parses_yaml_mappings() {
    let value = parse_structured_value(Path::new("app.yml"), "runtime:\n  version: 1\n").unwrap();
    assert_eq!(value["runtime"]["version"].as_i64(), Some(1));
}

#[test]
fn parses_jsonc_comments_and_trailing_commas() {
    let source = "{\n  // keep plugins\n  \"plugins\": [\"react\"],\n}\n";
    let value = parse_structured_value(Path::new(".oxlintrc.json"), source).unwrap();
    let plugins = value.get("plugins").and_then(Value::as_sequence).unwrap();
    assert_eq!(plugins[0].as_str(), Some("react"));
}

#[test]
fn jsonc_parse_errors_are_diagnostics() {
    let error = parse_structured_value(Path::new("broken.json"), "{").unwrap_err();
    assert!(error.contains("failed to parse JSONC"), "{error}");
}

#[test]
fn yaml_parse_errors_are_diagnostics() {
    let error =
        parse_structured_value(Path::new("broken.yml"), "runtime:\n  version: [\n").unwrap_err();
    assert!(error.contains("failed to parse YAML"), "{error}");
}

#[test]
fn jsonc_path_detection_covers_json_extensions() {
    assert!(is_jsonc_path(Path::new("a.json")));
    assert!(is_jsonc_path(Path::new("a.JSONC")));
    assert!(!is_jsonc_path(Path::new("a.yml")));
}

#[test]
fn converts_json_number_shapes() {
    let source = r#"{ "i": 2, "u": 1, "f": 1.5, "ok": true, "empty": null }"#;
    let value = parse_structured_value(Path::new("n.json"), source).unwrap();
    assert_eq!(value["i"].as_i64(), Some(2));
    assert!(value["ok"].as_bool().unwrap());
    assert!(value["empty"].is_null());
    assert!(value["f"].as_f64().unwrap() > 1.0);
}

#[test]
fn converts_json_integers_that_do_not_fit_i64() {
    let source = r#"{ "u": 18446744073709551615 }"#;
    let value = parse_structured_value(Path::new("n.json"), source).unwrap();
    assert!(value["u"].as_i64().is_none());
    assert_eq!(value["u"].as_u64(), Some(u64::MAX));
}
