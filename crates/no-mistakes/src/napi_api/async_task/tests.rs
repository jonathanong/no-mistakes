#[test]
fn napi_error_preserves_the_anyhow_chain() {
    let error = anyhow::anyhow!("outer").context("context");
    let napi_error = super::to_napi_error(error);
    assert!(napi_error.reason.contains("context: outer"));
}
