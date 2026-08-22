use super::*;
use serde_json::{json, Value};

#[test]
fn test_json_arg_accepts_value_and_string_forms() {
    let owned = json!({"root": "."});
    assert_eq!(test_json_arg(owned.clone()), owned);
    assert_eq!(test_json_arg(&owned), owned);

    let text = r#"{"root":"."}"#.to_string();
    let parsed: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(test_json_arg(text.clone()), parsed);
    assert_eq!(test_json_arg(&text), parsed);
    assert_eq!(test_json_arg(text.as_str()), parsed);
}

#[test]
#[should_panic(expected = "not valid json {{{")]
fn test_json_arg_panics_on_invalid_owned_string() {
    let _ = test_json_arg("not valid json {{{".to_string());
}

#[test]
#[should_panic(expected = "not valid json {{{")]
fn test_json_arg_panics_on_invalid_string_ref() {
    let text = "not valid json {{{".to_string();
    let _ = test_json_arg(&text);
}

#[test]
#[should_panic(expected = "not valid json {{{")]
fn test_json_arg_panics_on_invalid_str() {
    let _ = test_json_arg("not valid json {{{");
}
