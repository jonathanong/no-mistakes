use super::*;
use napi::Task;

#[test]
fn ordinary_outputs_and_errors_remain_unwrapped() {
    assert_eq!(finish(Ok("[1,2]".into()), None).unwrap().as_ref(), b"[1,2]");
    let error = finish(Err(napi::Error::from_reason("original error")), None)
        .err()
        .unwrap();
    assert_eq!(error.reason, "original error");
    let request = DiagnosticRequestGuard::extract(
        &mut serde_json::json!({"diagnosticsId":"progress-only-finish"}),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        finish(Ok("{}".into()), Some(&request)).unwrap().as_ref(),
        b"{}"
    );
}

#[test]
fn verbose_transport_preserves_array_results_and_authoritative_errors() {
    let request =
        DiagnosticRequestGuard::extract(&mut serde_json::json!({"__collectDiagnostics":true}))
            .unwrap()
            .unwrap();
    let output = finish(Ok("[1,2]".into()), Some(&request)).unwrap();
    let json: Value = serde_json::from_slice(output.as_ref()).unwrap();
    assert_eq!(json["result"], serde_json::json!([1, 2]));
    assert_eq!(json["diagnostics"]["work"]["output.renders"], 1);
    let error = finish(
        Err(napi::Error::from_reason("original error")),
        Some(&request),
    )
    .err()
    .unwrap();
    let (message, diagnostics) = error.reason.split_once(DIAGNOSTICS_JSON_MARKER).unwrap();
    assert_eq!(message, "original error");
    let diagnostics: Value = serde_json::from_str(diagnostics).unwrap();
    assert_eq!(diagnostics["work"]["output.errors"], 1);
    assert!(finish(Ok("invalid JSON".into()), Some(&request))
        .err()
        .unwrap()
        .reason
        .contains(DIAGNOSTICS_JSON_MARKER));
}

#[test]
fn source_batches_get_structured_diagnostics_without_changing_their_input() {
    let output = execute_pure(
        serde_json::json!({
            "__collectDiagnostics": true, "__diagnosticsInput": [1,2]
        }),
        |input| {
            assert_eq!(input, serde_json::json!([1, 2]));
            Ok(input.to_string())
        },
    )
    .unwrap();
    let result: Value = serde_json::from_slice(output.as_ref()).unwrap();
    assert_eq!(result["result"], serde_json::json!([1, 2]));
    assert!(result["diagnostics"]["timings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|timing| timing["label"] == "analysis"));
    assert!(execute_pure(
        serde_json::json!({"__collectDiagnostics":"yes"}),
        |_| unreachable!()
    )
    .is_err());
}

#[test]
fn both_native_transports_reuse_saved_analysis_and_return_diagnostics() {
    use crate::napi_api::async_task::{JsonTask, JsonValueTask};
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/diagnostics/public-output-parity");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let request = serde_json::json!({"root":fixture.path(), "__collectDiagnostics":true});
    let input = || Buffer::from(serde_json::to_vec(&request).unwrap());
    let mut ordinary = JsonValueTask::new(
        Buffer::from(serde_json::to_vec(&serde_json::json!({"root":fixture.path()})).unwrap()),
        crate::napi_api::check_json_impl,
    );
    let expected: Value = serde_json::from_slice(ordinary.compute().unwrap().as_ref()).unwrap();
    let mut string = JsonTask::new(input(), |options| {
        crate::napi_api::check_json_impl(serde_json::from_str(&options).map_err(to_napi_error)?)
    });
    let mut value = JsonValueTask::new(input(), crate::napi_api::check_json_impl);
    for output in [string.compute().unwrap(), value.compute().unwrap()] {
        let report: Value = serde_json::from_slice(output.as_ref()).unwrap();
        assert_eq!(report["result"], expected);
        assert_eq!(report["diagnostics"]["work"]["output.renders"], 1);
        assert!(report["diagnostics"]["timings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|timing| timing["label"] == "invocation.dispatch"));
        assert!(crate::diagnostics::current().is_none());
    }
}
