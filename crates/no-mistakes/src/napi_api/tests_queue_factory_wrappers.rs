use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn configured_queue_wrappers_share_api_reports_edges_and_one_parse() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/configured-enqueue-wrappers/fixture")
        .canonicalize()
        .unwrap();
    let options = || crate::napi_api::options::test_json_arg(json!({"root": root}).to_string());
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
    crate::ast::begin_parse_count(&root);
    let batch = super::analyze_project::analyze_project_json_impl(
        crate::napi_api::options::test_json_arg(json!({
            "root": root,
            "reports": [{"type": "queues"}, {"type": "queueEdges"}, {"type": "queueCheck"}, {"type": "check"}]
        }).to_string()),
    ).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
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
