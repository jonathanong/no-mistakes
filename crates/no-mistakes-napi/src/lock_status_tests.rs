use super::async_task::{to_invocation_napi_error, ERROR_JSON_MARKER};
use super::lock_status_json_impl;
use serde_json::json;

#[test]
fn lock_status_reports_state_and_lock_file_and_rejects_unknown_options() {
    const CHILD: &str = "NO_MISTAKES_LOCK_STATUS_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        // The test lock path uses this process's PID; parallel unit tests can hold it.
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "napi_api::lock_status_tests::lock_status_reports_state_and_lock_file_and_rejects_unknown_options",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated lock status test failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let report: serde_json::Value =
        serde_json::from_str(&lock_status_json_impl(json!({})).unwrap()).unwrap();
    assert_eq!(report["state"], "free");
    assert!(report["lockFile"].as_str().unwrap().ends_with(".lock"));
    assert!(report["holder"].is_null());
    assert!(lock_status_json_impl(json!({ "unknownField": true })).is_err());
}

#[test]
fn invocation_errors_append_a_machine_readable_report_to_the_message() {
    let plain = to_invocation_napi_error(anyhow::anyhow!("boom"));
    assert_eq!(plain.reason, "boom");
    let io: anyhow::Error = std::io::Error::from(std::io::ErrorKind::TimedOut).into();
    let structured = to_invocation_napi_error(io);
    let (message, report) = structured.reason.split_once(ERROR_JSON_MARKER).unwrap();
    assert!(!message.is_empty());
    let report: serde_json::Value = serde_json::from_str(report).unwrap();
    assert_eq!(report["kind"], "deadline");
    assert_eq!(report["exitCode"], 124);
}
