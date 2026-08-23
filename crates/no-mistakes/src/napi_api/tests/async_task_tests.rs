use napi::bindgen_prelude::Buffer;
use napi::{Env, Task};

use super::super::async_task::{JsonTask, JsonValueTask, VersionTask};

fn json_buffer(json: &str) -> Buffer {
    Buffer::from(json.as_bytes())
}

fn utf8(output: Buffer) -> String {
    String::from_utf8(output.to_vec()).unwrap()
}

fn echo_task(input: String) -> napi::Result<String> {
    Ok(format!("echo:{input}"))
}

fn failing_task(_input: String) -> napi::Result<String> {
    Err(napi::Error::from_reason("task failed"))
}

fn parse_cache_task(_input: String) -> napi::Result<String> {
    Ok(crate::ast::request_parse_cache_active().to_string())
}

fn value_task(input: serde_json::Value) -> napi::Result<String> {
    Ok(format!(
        "{}:{}",
        input["root"].as_str().unwrap(),
        crate::ast::request_parse_cache_active()
    ))
}

#[test]
fn async_json_task_runs_on_task_interface() {
    let mut task = JsonTask::new(json_buffer("{}"), echo_task);

    assert_eq!(utf8(task.compute().unwrap()), "echo:{}");
    assert_eq!(
        utf8(
            task.resolve(Env::from_raw(std::ptr::null_mut()), json_buffer("done"))
                .unwrap()
        ),
        "done"
    );

    let mut task = JsonTask::new(json_buffer("{}"), failing_task);
    let error = task.compute().err().expect("echo task should fail");
    assert!(error.reason.contains("task failed"));

    let mut task = JsonTask::new(json_buffer("{}"), parse_cache_task);
    assert_eq!(utf8(task.compute().unwrap()), "true");
    assert!(!crate::ast::request_parse_cache_active());
}

#[test]
fn async_json_task_rejects_non_utf8_options() {
    let mut task = JsonTask::new(Buffer::from(vec![0xff, 0xfe]), echo_task);
    let error = task.compute().err().expect("non-UTF-8 options should fail");
    assert!(error.reason.contains("options JSON must be UTF-8"));
}

#[test]
fn async_json_value_task_parses_options_once() {
    let mut task = JsonValueTask::new(json_buffer(r#"{"timeout":4,"root":"fixture"}"#), value_task);

    assert_eq!(utf8(task.compute().unwrap()), "fixture:true");
    assert_eq!(
        utf8(
            task.resolve(Env::from_raw(std::ptr::null_mut()), json_buffer("done"))
                .unwrap()
        ),
        "done"
    );
    assert!(!crate::ast::request_parse_cache_active());
}

#[test]
fn async_version_task_runs_on_task_interface() {
    let mut task = VersionTask;

    assert_eq!(task.compute().unwrap(), env!("CARGO_PKG_VERSION"));
    assert_eq!(
        task.resolve(Env::from_raw(std::ptr::null_mut()), "0.0.0".to_string())
            .unwrap(),
        "0.0.0"
    );
}

#[test]
fn async_json_task_installs_the_global_rayon_pool() {
    let mut task = JsonTask::new(json_buffer("{}"), echo_task);
    task.compute().unwrap();
    assert!(rayon::current_num_threads() >= 1);
}
