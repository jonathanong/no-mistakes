use super::{json_pretty, json_string, json_value, write_json, yaml_string};

#[test]
fn write_json_emits_compact_object_and_trailing_newline() {
    let mut buf = Vec::new();
    write_json(&mut buf, &serde_json::json!({ "ok": true }));
    assert_eq!(
        buf,
        br#"{"ok":true}
"#
    );
}

#[test]
fn json_and_yaml_helpers_serialize_rust_structs() {
    let value = serde_json::json!({ "ok": true });
    assert_eq!(json_string(&value), r#"{"ok":true}"#);
    assert_eq!(json_value(&value), value);
    assert!(json_pretty(&value).contains("\"ok\": true"));
    assert!(yaml_string(&value).contains("ok:"));
}
