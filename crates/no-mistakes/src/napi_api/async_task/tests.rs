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

// Rayon cannot reset its global pool, so each case must execute in a fresh process.
#[test]
fn first_invocation_jobs_subprocess() {
    let Ok(task) = std::env::var("NO_MISTAKES_NAPI_JOBS_TASK") else {
        return;
    };
    use napi::Task;
    fn run_string(options: String) -> napi::Result<String> {
        let _: crate::napi_api::options::ProjectOptions = serde_json::from_str(&options)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        Ok(rayon::current_num_threads().to_string())
    }
    fn run_value(options: serde_json::Value) -> napi::Result<String> {
        let _: crate::napi_api::options::ProjectOptions = serde_json::from_value(options)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        Ok(rayon::current_num_threads().to_string())
    }
    let compute = |payload: Vec<u8>| {
        let buffer = napi::bindgen_prelude::Buffer::from(payload);
        if task == "string" {
            super::JsonTask::new(buffer, run_string).compute()
        } else {
            super::JsonValueTask::new(buffer, run_value).compute()
        }
    };
    if std::env::var_os("NO_MISTAKES_NAPI_JOBS_INVALID_FIRST").is_some() {
        assert!(compute(b"{".to_vec()).is_err());
        assert!(compute(vec![0xff]).is_err());
        assert!(compute(br#"{"jobs":-1}"#.to_vec()).is_err());
    }
    if std::env::var_os("NO_MISTAKES_NAPI_JOBS_INVALID_COMMAND_FIRST").is_some() {
        // Invocation controls are valid and the lock is acquired before the
        // command rejects root. That rejected request still selects one worker.
        let error = compute(br#"{"jobs":1,"root":123}"#.to_vec())
            .err()
            .expect("command-specific root validation rejects a number");
        assert!(error.reason.contains("invalid type"), "{}", error.reason);
    }
    if std::env::var_os("NO_MISTAKES_NAPI_SQL_BATCH_FIRST").is_some() {
        let sql = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/postgres-facts/source/schema.sql"
        ));
        let input = serde_json::json!([
            {"sql": sql, "fileName": "first.sql"},
            {"sql": sql, "fileName": "second.sql"},
        ]);
        let output = crate::napi_api::postgres_source::parse_postgres_sql_json_impl(input).unwrap();
        let facts: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(facts[0]["fileName"], "first.sql");
        assert_eq!(facts[1]["fileName"], "second.sql");
        assert_eq!(facts[0]["schemaVersion"], 1);
    }
    let payload = std::env::var("NO_MISTAKES_NAPI_JOBS_PAYLOAD").unwrap();
    let expected = std::env::var("NO_MISTAKES_NAPI_JOBS_EXPECTED").unwrap();
    let expected = if expected == "cpu" {
        num_cpus::get().to_string()
    } else {
        expected
    };
    let output = compute(payload.into_bytes()).unwrap();
    assert_eq!(std::str::from_utf8(output.as_ref()).unwrap(), expected);
    // A later request must preserve the pool selected by the first request that reaches pool initialization.
    let output = compute(br#"{"jobs":2}"#.to_vec()).unwrap();
    assert_eq!(std::str::from_utf8(output.as_ref()).unwrap(), expected);
}

#[test]
fn first_invocation_jobs_override_environment_and_preserve_defaults() {
    for task in ["string", "value"] {
        for (payload, environment, expected, invalid_first) in [
            (r#"{"jobs":1}"#, "4", "1", false),
            (r#"{"jobs":4}"#, "1", "4", false),
            ("{}", "3", "3", false),
            (r#"{"jobs":0}"#, "3", "3", false),
            ("{}", "", "cpu", false),
            (r#"{"jobs":0}"#, "", "cpu", false),
            (r#"{"jobs":1}"#, "4", "1", true),
        ] {
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--exact",
                    "napi_api::async_task::tests::first_invocation_jobs_subprocess",
                    "--nocapture",
                ])
                .env("RAYON_NUM_THREADS", environment)
                .env("NO_MISTAKES_NAPI_JOBS_TASK", task)
                .env("NO_MISTAKES_NAPI_JOBS_PAYLOAD", payload)
                .env("NO_MISTAKES_NAPI_JOBS_EXPECTED", expected)
                .env_remove("NO_MISTAKES_NAPI_SQL_BATCH_FIRST")
                .env_remove("NO_MISTAKES_NAPI_JOBS_INVALID_FIRST")
                .env_remove("NO_MISTAKES_NAPI_JOBS_INVALID_COMMAND_FIRST");
            if invalid_first {
                command.env("NO_MISTAKES_NAPI_JOBS_INVALID_FIRST", "1");
            }
            if environment.is_empty() {
                command.env_remove("RAYON_NUM_THREADS");
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "{task}, {payload}, RAYON_NUM_THREADS={environment}, invalid_first={invalid_first}:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
        }
    }
}

#[test]
fn rejected_command_options_still_select_the_global_pool() {
    for task in ["string", "value"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "napi_api::async_task::tests::first_invocation_jobs_subprocess",
                "--nocapture",
            ])
            .env("RAYON_NUM_THREADS", "3")
            .env("NO_MISTAKES_NAPI_JOBS_TASK", task)
            .env("NO_MISTAKES_NAPI_JOBS_INVALID_COMMAND_FIRST", "1")
            .env_remove("NO_MISTAKES_NAPI_SQL_BATCH_FIRST")
            .env_remove("NO_MISTAKES_NAPI_JOBS_INVALID_FIRST")
            .env("NO_MISTAKES_NAPI_JOBS_PAYLOAD", r#"{"jobs":4}"#)
            .env("NO_MISTAKES_NAPI_JOBS_EXPECTED", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{task}:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
}

#[test]
fn sql_source_batch_does_not_pin_repository_analysis_worker_count() {
    for task in ["string", "value"] {
        for (payload, environment, expected) in
            [(r#"{"jobs":1}"#, "4", "1"), (r#"{"jobs":4}"#, "1", "4")]
        {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "napi_api::async_task::tests::first_invocation_jobs_subprocess",
                    "--nocapture",
                ])
                .env("RAYON_NUM_THREADS", environment)
                .env("NO_MISTAKES_NAPI_JOBS_TASK", task)
                .env("NO_MISTAKES_NAPI_JOBS_PAYLOAD", payload)
                .env("NO_MISTAKES_NAPI_JOBS_EXPECTED", expected)
                .env("NO_MISTAKES_NAPI_SQL_BATCH_FIRST", "1")
                .env_remove("NO_MISTAKES_NAPI_JOBS_INVALID_COMMAND_FIRST")
                .env_remove("NO_MISTAKES_NAPI_JOBS_INVALID_FIRST")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{task}, jobs={expected}, env={environment}:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
        }
    }
}

#[test]
fn concurrent_progress_polling_does_not_wait_for_the_invocation_lock_and_cleans_up_errors() {
    use napi::Task;
    use std::sync::{mpsc, Mutex, OnceLock};
    use std::time::Duration;
    type Controls = (mpsc::Sender<()>, mpsc::Receiver<()>);
    static CONTROLS: OnceLock<Mutex<Option<Controls>>> = OnceLock::new();
    fn operation(options: serde_json::Value) -> napi::Result<String> {
        let (ready, release) = CONTROLS.get().unwrap().lock().unwrap().take().unwrap();
        assert!(options.get("diagnosticsId").is_none());
        crate::perf_trace::trace("analysis.held", || {
            ready.send(()).unwrap();
            release.recv_timeout(Duration::from_secs(3)).unwrap();
            if options["fail"] == true {
                Err(napi::Error::from_reason("analysis error"))
            } else {
                Ok("{}".to_string())
            }
        })
    }
    fn string_operation(options: String) -> napi::Result<String> {
        operation(serde_json::from_str(&options).unwrap())
    }
    for string in [false, true] {
        for fail in [false, true] {
            let (ready, started) = mpsc::channel();
            let (release, resume) = mpsc::channel();
            *CONTROLS.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some((ready, resume));
            let worker = std::thread::spawn(move || {
                let options =
                    serde_json::json!({"diagnosticsId":"task-concurrency-test", "fail":fail});
                let buffer =
                    napi::bindgen_prelude::Buffer::from(serde_json::to_vec(&options).unwrap());
                if string {
                    super::JsonTask::new(buffer, string_operation).compute()
                } else {
                    super::JsonValueTask::new(buffer, operation).compute()
                }
            });
            started.recv_timeout(Duration::from_secs(3)).unwrap();
            // Exercise the snapshot implementation off the held analysis thread.
            // The public binding uses a separate deferred-promise worker,
            // also covering libuv starvation in the real-addon tests.
            let input =
                napi::bindgen_prelude::Buffer::from(br#"{"id":"task-concurrency-test"}"#.to_vec());
            let output =
                super::pure::PureJsonTask::new(input, crate::napi_api::diagnostics_snapshot_impl)
                    .compute()
                    .unwrap();
            let snapshot: serde_json::Value = serde_json::from_slice(&output).unwrap();
            assert!(snapshot["active"]
                .as_array()
                .unwrap()
                .iter()
                .any(|span| span["phase"]
                    == serde_json::json!(["invocation.dispatch", "analysis.held"])));
            release.send(()).unwrap();
            assert_eq!(worker.join().unwrap().is_err(), fail);
            assert!(crate::diagnostics::progress_for_request("task-concurrency-test").is_none());
        }
    }
}
