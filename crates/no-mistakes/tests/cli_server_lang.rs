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
fn kafka_language_edges_agree_across_queues_and_dependents() {
    let root = lang_fixture("kafka-languages");
    let root = root.to_str().unwrap();
    let queues = run(&["queues", "--root", root, "--format", "json", "edges"]);
    assert!(queues.status.success(), "{}", stdout(&queues));
    let queue_json: serde_json::Value = serde_json::from_str(&stdout(&queues)).unwrap();
    let mut jobs = std::collections::BTreeSet::new();
    for edge in queue_json.as_array().unwrap() {
        for endpoint in [edge["from"].as_str(), edge["to"].as_str()]
            .into_iter()
            .flatten()
        {
            if let Some((_, job)) = endpoint.rsplit_once('#') {
                jobs.insert(job.to_string());
            }
        }
    }
    assert_eq!(
        jobs,
        std::collections::BTreeSet::from([
            "billing:mail.welcome".to_string(),
            "mail:mail.welcome".to_string(),
            "notifications:mail.welcome".to_string(),
        ])
    );
    let dependents = run(&[
        "dependents",
        "clusters/billing/consume.go",
        "go/consume.go",
        "--root",
        root,
        "--relationship",
        "queue",
        "--depth",
        "2",
        "--format",
        "json",
    ]);
    assert!(dependents.status.success(), "{}", stdout(&dependents));
    let dependent_json = stdout(&dependents);
    assert!(dependent_json.contains("billing:mail.welcome"));
    assert!(dependent_json.contains("mail:mail.welcome"));
    assert!(!dependent_json.contains("notifications:mail.welcome"));
    assert!(!dependent_json.contains('*'));
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
    assert!(json
        .as_array()
        .unwrap()
        .iter()
        .any(|edge| { edge["kind"] == "queue-enqueue" || edge["kind"] == "queue-worker" }));
}
