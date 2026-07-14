use napi::{Env, Task};

use super::super::async_task::{JsonTask, VersionTask};

fn echo_task(input: String) -> napi::Result<String> {
    Ok(format!("echo:{input}"))
}

fn failing_task(_input: String) -> napi::Result<String> {
    Err(napi::Error::from_reason("task failed"))
}

fn parse_cache_task(_input: String) -> napi::Result<String> {
    Ok(crate::ast::request_parse_cache_active().to_string())
}

#[test]
fn async_json_task_runs_on_task_interface() {
    let mut task = JsonTask::new("{}".to_string(), echo_task);

    assert_eq!(task.compute().unwrap(), "echo:{}");
    assert_eq!(
        task.resolve(Env::from_raw(std::ptr::null_mut()), "done".to_string())
            .unwrap(),
        "done"
    );

    let mut task = JsonTask::new("{}".to_string(), failing_task);
    assert!(task.compute().unwrap_err().reason.contains("task failed"));

    let mut task = JsonTask::new("{}".to_string(), parse_cache_task);
    assert_eq!(task.compute().unwrap(), "true");
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
