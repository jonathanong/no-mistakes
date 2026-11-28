use super::*;
fn echo(value: serde_json::Value) -> napi::Result<String> {
    Ok(value.to_string())
}
#[test]
fn owned_request_parsing_preserves_json_and_rejects_utf8_json_and_callback_errors() {
    assert_eq!(
        run_json(br#"{"id":"live"}"#, echo).unwrap(),
        r#"{"id":"live"}"#
    );
    assert!(run_json(&[0xff], echo)
        .unwrap_err()
        .reason
        .contains("UTF-8"));
    assert!(run_json(b"{", echo).is_err());
    fn reject(_: serde_json::Value) -> napi::Result<String> {
        Err(napi::Error::from_reason("invalid request"))
    }
    assert_eq!(
        run_json(b"{}", reject).unwrap_err().reason,
        "invalid request"
    );
    fn panic(_: serde_json::Value) -> napi::Result<String> {
        panic!("poll failed");
    }
    assert_eq!(
        run_json(b"{}", panic).unwrap_err().reason,
        "progress polling panicked"
    );
}
#[test]
fn a_panicking_job_does_not_stop_other_polling_jobs() {
    let (sender, ready) = mpsc::channel();
    let worker = Executor::start(2, move |job| {
        if job == 0 {
            panic!("environment closed");
        }
        sender.send(job).unwrap();
    });
    worker.submit(0).unwrap();
    worker.submit(1).unwrap();
    assert_eq!(
        ready
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap(),
        1
    );
}
