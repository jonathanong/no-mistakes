use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::Value;

use crate::napi_api::{dependents_json_impl, queue_edges_json_impl};
use crate::queue::{analyze_project, Edge, EdgeKind};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/lang-frontends/kafka-languages")
}

fn report() -> crate::queue::ProjectReport {
    analyze_project(&fixture(), None, &[]).unwrap()
}

fn job_of(endpoint: &str) -> Option<&str> {
    endpoint.rsplit_once('#').map(|(_, job)| job)
}

fn edge_jobs(edges: &[Edge]) -> BTreeSet<String> {
    edges
        .iter()
        .filter_map(|edge| match edge.kind {
            EdgeKind::QueueEnqueue => job_of(&edge.to),
            EdgeKind::QueueWorker => job_of(&edge.from),
        })
        .map(str::to_string)
        .collect()
}

fn assert_pair(produce: &str, consume: &str) {
    let report = report();
    assert!(
        report.edges.iter().any(|edge| {
            edge.kind == EdgeKind::QueueEnqueue
                && edge.from.ends_with(produce)
                && edge.to.ends_with("#mail:mail.welcome")
        }),
        "{produce}"
    );
    assert!(
        report.edges.iter().any(|edge| {
            edge.kind == EdgeKind::QueueWorker
                && edge.to.ends_with(consume)
                && edge.from.ends_with("#mail:mail.welcome")
        }),
        "{consume}"
    );
}

#[test]
fn go_kafka_fixture_projects_topic_edges() {
    assert_pair("go/produce.go", "go/consume.go");
}

#[test]
fn rust_kafka_fixture_projects_topic_edges() {
    assert_pair("rust/produce.rs", "rust/consume.rs");
}

#[test]
fn ruby_kafka_fixture_projects_topic_edges() {
    assert_pair("ruby/produce.rb", "ruby/consume.rb");
}

#[test]
fn php_kafka_fixture_projects_topic_edges() {
    assert_pair("php/produce.php", "php/consume.php");
}

#[test]
fn java_kafka_fixture_projects_topic_edges() {
    assert_pair("java/produce.java", "java/consume.java");
}

#[test]
fn kotlin_kafka_fixture_projects_topic_edges() {
    assert_pair("kotlin/produce.kt", "kotlin/consume.kt");
}

#[test]
fn elixir_kafka_fixture_projects_topic_edges() {
    assert_pair("elixir/produce.ex", "elixir/consume.ex");
}

#[test]
fn dart_kafka_fixture_projects_topic_edges() {
    assert_pair("dart/produce.dart", "dart/consume.dart");
}

#[test]
fn two_clusters_reuse_one_topic_and_stay_distinct() {
    let jobs = edge_jobs(&report().edges);
    assert!(jobs.contains("billing:mail.welcome"));
    assert!(jobs.contains("notifications:mail.welcome"));
    assert_eq!(
        jobs,
        BTreeSet::from([
            "billing:mail.welcome".to_string(),
            "mail:mail.welcome".to_string(),
            "notifications:mail.welcome".to_string(),
        ])
    );
}

fn json_jobs(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(job) = map.get("job").and_then(Value::as_str) {
                out.insert(job.to_string());
            }
            for child in map.values() {
                json_jobs(child, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                json_jobs(item, out);
            }
        }
        _ => {}
    }
}

#[test]
fn queue_edges_and_dependents_agree_on_kafka_topic_identity() {
    let root = fixture();
    let expected = edge_jobs(&report().edges);
    let napi = queue_edges_json_impl(serde_json::json!({ "root": root.display().to_string() }))
        .expect("queue edges");
    let edges: Value = serde_json::from_str(&napi).expect("queue edge json");
    let mut napi_jobs = BTreeSet::new();
    for edge in edges.as_array().expect("edges") {
        for endpoint in [edge["from"].as_str(), edge["to"].as_str()]
            .into_iter()
            .flatten()
        {
            if let Some(job) = job_of(endpoint) {
                napi_jobs.insert(job.to_string());
            }
        }
    }
    assert_eq!(napi_jobs, expected);

    let dependents = dependents_json_impl(serde_json::json!({
        "root": root.display().to_string(),
        "files": ["clusters/billing/consume.go", "go/consume.go"],
        "relationships": ["queue"],
        "depth": 2,
    }))
    .expect("dependents");
    let mut jobs = BTreeSet::new();
    json_jobs(
        &serde_json::from_str::<Value>(&dependents).unwrap(),
        &mut jobs,
    );
    assert!(jobs.contains("billing:mail.welcome"));
    assert!(jobs.contains("mail:mail.welcome"));
    assert!(!jobs.contains("notifications:mail.welcome"));
}
