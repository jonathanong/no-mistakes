#[test]
fn napi_error_preserves_the_anyhow_chain() {
    let error = anyhow::anyhow!("outer").context("context");
    let napi_error = super::to_napi_error(error);
    assert!(napi_error.reason.contains("context: outer"));
}

#[test]
fn utf8_json_accepts_valid_utf8_without_copying_into_owned_string_on_check() {
    let json = br#"{"root":"."}"#;
    let buffer = napi::bindgen_prelude::Buffer::from(json.to_vec());
    let decoded = super::utf8_json(&buffer).expect("valid UTF-8 options");
    assert_eq!(decoded, r#"{"root":"."}"#);
}

#[test]
fn utf8_json_rejects_invalid_utf8() {
    let buffer = napi::bindgen_prelude::Buffer::from(vec![0xff, 0xfe]);
    let error = super::utf8_json(&buffer).expect_err("invalid UTF-8");
    assert!(error.reason.contains("UTF-8"));
}
