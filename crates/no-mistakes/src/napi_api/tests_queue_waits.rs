use serde_json::json;
use std::path::PathBuf;

#[test]
fn queue_check_json_recognizes_wait_values_and_respects_alias_scopes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/wait-forms/fixture");
    let result = super::project::queue_check_json_impl(json!({ "root": root })).unwrap();
    let findings: serde_json::Value = serde_json::from_str(&result).unwrap();
    let findings = findings.as_array().unwrap();
    assert_eq!(findings.len(), 16, "{findings:?}");
    let mut positive: Vec<_> = findings
        .iter()
        .filter(|finding| {
            finding["file"].as_str().unwrap().replace('\\', "/") == "api/routes/positive.ts"
        })
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect();
    positive.sort_unstable();
    assert_eq!(positive, [3, 4, 5, 7, 8, 9, 10, 11, 13, 15, 16, 17]);
    assert!(findings.iter().all(|finding| {
        finding["kind"] == "enqueue-and-wait"
            && finding["file"].as_str().unwrap().replace('\\', "/") != "api/routes/negative.ts"
            && finding["file"].as_str().unwrap().replace('\\', "/") != "api/worker.ts"
    }));
}
