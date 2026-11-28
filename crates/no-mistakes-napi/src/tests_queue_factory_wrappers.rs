use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn configured_queue_wrappers_share_api_reports_edges_and_one_parse() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/configured-enqueue-wrappers/fixture")
        .canonicalize()
        .unwrap();
    let options = || crate::options::test_json_arg(json!({"root": root}).to_string());
    let standalone = [
        super::queues_json_impl(options()).unwrap(),
        super::queue_edges_json_impl(options()).unwrap(),
        super::queue_check_json_impl(options()).unwrap(),
    ]
    .map(|s| serde_json::from_str::<Value>(&s).unwrap());
    for field in ["producers", "workers"] {
        let sites = standalone[0][field].as_array().unwrap();
        let key = |site: &Value| {
            (
                site["file"].as_str().unwrap().to_owned(),
                site["line"].as_u64().unwrap(),
            )
        };
        assert!(
            sites.windows(2).all(|pair| key(&pair[0]) <= key(&pair[1])),
            "{field}: {sites:?}"
        );
    }
    let jobs: BTreeSet<_> = standalone[0]["producers"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["job"].as_str())
        .collect();
    assert_eq!(
        jobs,
        BTreeSet::from([
            "objectTask",
            "directTask",
            "noDefaultsTask",
            "returnedTask",
            "definitionTask",
            "bulkTask",
        ])
    );
    assert!(!standalone[1].as_array().unwrap().is_empty());
    assert!(standalone[2]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["kind"] != "unmatched-producer" && f["kind"] != "unmatched-worker"));
    no_mistakes::ast::begin_parse_count(&root);
    let batch = super::analyze_project::analyze_project_json_impl(
        crate::options::test_json_arg(json!({
            "root": root,
            "reports": [{"type": "queues"}, {"type": "queueEdges"}, {"type": "queueCheck"}, {"type": "check"}]
        }).to_string()),
    ).unwrap();
    let counts = no_mistakes::ast::finish_parse_count(&root);
    let batch: Value = serde_json::from_str(&batch).unwrap();
    for (i, expected) in standalone.iter().enumerate() {
        assert_eq!(&batch["reports"][i]["result"], expected);
    }
    assert_eq!(batch["reports"][3]["result"]["queues"], standalone[2]);
    assert!(!counts.is_empty());
    assert!(counts.values().all(|n| *n == 1), "{counts:?}");
    let edges = standalone[1].as_array().unwrap();
    for file in [
        "enqueues/object.ts",
        "enqueues/returned.ts",
        "enqueues/fixed.ts",
    ] {
        assert!(
            edges
                .iter()
                .any(|e| e["from"] == file && e["kind"] == "queue-enqueue"),
            "{file}: {edges:?}"
        );
    }
    for job in jobs {
        let target = edges
            .iter()
            .find(|e| {
                e["kind"] == "queue-enqueue"
                    && e["to"]
                        .as_str()
                        .is_some_and(|name| name.ends_with(&format!("#{job}")))
            })
            .unwrap();
        assert!(
            edges.iter().any(|e| e["kind"] == "queue-worker"
                && e["from"] == target["to"]
                && e["to"] == "workers/worker.ts"),
            "{job}: {edges:?}"
        );
    }
    assert!(edges.iter().all(|e| e["from"] != "enqueues/abstain.ts"));
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
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/lang-frontends/kafka-languages");
    let report = no_mistakes::queue::analyze_project(&root, None, &[]).unwrap();
    fn job_of(endpoint: &str) -> Option<&str> {
        endpoint.rsplit_once('#').map(|(_, job)| job)
    }
    let expected: BTreeSet<String> = report
        .edges
        .iter()
        .filter_map(|edge| match edge.kind {
            no_mistakes::queue::EdgeKind::QueueEnqueue => job_of(&edge.to),
            no_mistakes::queue::EdgeKind::QueueWorker => job_of(&edge.from),
        })
        .map(str::to_string)
        .collect();
    let napi = super::queue_edges_json_impl(crate::options::test_json_arg(
        json!({ "root": root.display().to_string() }).to_string(),
    ))
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

    let dependents = super::dependents_json_impl(crate::options::test_json_arg(
        json!({
            "root": root.display().to_string(),
            "files": ["clusters/billing/consume.go", "go/consume.go"],
            "relationships": ["queue"],
            "depth": 2,
        })
        .to_string(),
    ))
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
