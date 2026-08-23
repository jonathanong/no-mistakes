use super::write_json;

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
