use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn lang_fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/lang-frontends")
            .join(name),
    )
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

#[test]
fn server_routes_json_lists_go_http_literals() {
    let root = lang_fixture("go-http");
    let output = run(&[
        "server",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
        "routes",
    ]);
    assert!(output.status.success(), "{}", stdout(&output));
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(json
        .as_array()
        .unwrap()
        .iter()
        .any(|route| route["route"] == "/health"));
}

#[test]
fn queues_edges_json_lists_celery_jobs() {
    let root = lang_fixture("python-celery-django");
    let output = run(&[
        "queues",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
        "edges",
    ]);
    assert!(output.status.success(), "{}", stdout(&output));
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert!(json.as_array().unwrap().iter().any(|edge| {
        edge["kind"] == "queue-enqueue" || edge["kind"] == "queue-worker"
    }));
}
