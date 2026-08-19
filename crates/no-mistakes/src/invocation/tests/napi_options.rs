use super::*;

#[test]
fn napi_options_default_and_strip_controls() {
    let (json, options) = extract_napi_options(
        r#"{"timeout":4,"lockTimeout":5,"failOnLock":true,"root":"."}"#.to_string(),
    )
    .unwrap();
    assert_eq!(options.timeout, Some(Duration::from_secs(4)));
    assert_eq!(options.lock_timeout, Some(Duration::from_secs(5)));
    assert!(options.fail_on_lock);
    assert_eq!(options.jobs, None);
    assert_eq!(
        serde_json::from_str::<Value>(&json).unwrap(),
        serde_json::json!({"root":"."})
    );
}

#[test]
fn napi_value_options_strip_controls_without_reserializing() {
    let (value, options) =
        extract_napi_options_value(r#"{"timeout":4,"root":".","reports":[]}"#.to_string()).unwrap();

    assert_eq!(options.timeout, Some(Duration::from_secs(4)));
    assert_eq!(value, serde_json::json!({"root":".","reports":[]}));
}

#[test]
fn napi_zero_and_null_disable_timeouts() {
    let (_, options) =
        extract_napi_options(r#"{"timeout":0,"lockTimeout":null}"#.to_string()).unwrap();
    assert_eq!(options.timeout, None);
    assert_eq!(options.lock_timeout, None);
}

#[test]
fn napi_missing_controls_use_defaults() {
    let (_, options) = extract_napi_options("{}".to_string()).unwrap();
    assert_eq!(options, InvocationOptions::default());
}

#[test]
fn napi_jobs_parses_non_negative_integer_or_null() {
    let (_, options) = extract_napi_options(r#"{"jobs":4}"#.to_string()).unwrap();
    assert_eq!(options.jobs, Some(4));
    let (_, options) = extract_napi_options(r#"{"jobs":0}"#.to_string()).unwrap();
    assert_eq!(options.jobs, Some(0));
    let (_, options) = extract_napi_options(r#"{"jobs":null}"#.to_string()).unwrap();
    assert_eq!(options.jobs, None);
}

#[test]
fn napi_controls_validate_types() {
    for json in [
        r#"{"timeout":-1}"#,
        r#"{"timeout":1.5}"#,
        r#"{"lockTimeout":"30"}"#,
        r#"{"failOnLock":1}"#,
        r#"{"jobs":-1}"#,
        r#"{"jobs":"4"}"#,
        "[]",
        "not-json",
    ] {
        assert!(extract_napi_options(json.to_string()).is_err(), "{json}");
    }
}
