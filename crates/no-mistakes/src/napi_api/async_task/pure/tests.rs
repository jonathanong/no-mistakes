use super::*;
#[test]
fn postgres_source_task_accepts_batches_without_repository_invocation_options() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/schema.sql"
    ));
    let input = serde_json::to_vec(&serde_json::json!([{"sql":sql}])).unwrap();
    let mut task = PureJsonTask::new(
        Buffer::from(input),
        crate::napi_api::postgres_source::parse_postgres_sql_json_impl,
    );
    let output = task.compute().unwrap();
    // resolve does not access its environment: pure transport returns the worker buffer.
    let output = task
        .resolve(Env::from_raw(std::ptr::null_mut()), output)
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(result[0]["schemaVersion"], 1);
    let mut invalid = PureJsonTask::new(
        Buffer::from(b"[".to_vec()),
        crate::napi_api::postgres_source::parse_postgres_sql_json_impl,
    );
    assert!(invalid.compute().is_err());
    let mut invalid_utf8 = PureJsonTask::new(
        Buffer::from(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/napi/derived-resolve-fact-parity/invalid-utf8.ts"
            ))
            .to_vec(),
        ),
        crate::napi_api::postgres_source::parse_postgres_sql_json_impl,
    );
    assert!(invalid_utf8.compute().is_err());
}
