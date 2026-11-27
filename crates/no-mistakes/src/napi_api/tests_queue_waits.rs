use serde_json::json;
use std::path::PathBuf;

#[test]
fn queue_check_json_recognizes_wait_values_and_respects_alias_scopes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/wait-forms/fixture");
    let result = super::queue_check_json_impl(json!({ "root": root })).unwrap();
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

#[test]
fn queue_check_json_exposes_wrapper_patterns_and_declaration_policy() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/wait-wrappers/fixture");
    let result = super::queue_check_json_impl(json!({
        "root": root
    }))
    .unwrap();
    let findings: serde_json::Value = serde_json::from_str(&result).unwrap();
    let findings = findings.as_array().unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding["kind"] == "enqueue-and-wait"));
    let declarations: Vec<_> = findings
        .iter()
        .filter(|finding| finding["kind"] == "enqueue-and-wait-declaration")
        .collect();
    let mut files: Vec<_> = declarations
        .iter()
        .map(|finding| finding["file"].as_str().unwrap().replace('\\', "/"))
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "api/routes/declaration-arrow.ts",
            "api/routes/declaration-expression.ts",
            "api/routes/declaration-function.ts",
            "api/routes/suppressed-declaration.ts",
        ]
    );
    assert!(declarations.iter().all(|finding| finding["message"]
        .as_str()
        .unwrap()
        .contains("enqueue without waiting")));
}

#[test]
fn queue_check_json_surfaces_diagnostics_when_opted_in_and_in_aggregate_check() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/worker-rate-limit-unresolved/fixture");
    let bare = super::queue_check_json_impl(json!({ "root": root })).unwrap();
    let bare: serde_json::Value = serde_json::from_str(&bare).unwrap();
    assert_eq!(bare, json!([]));

    let with_diagnostics = super::queue_check_json_impl(json!({
        "root": root,
        "includeDiagnostics": true,
    }))
    .unwrap();
    let with_diagnostics: serde_json::Value = serde_json::from_str(&with_diagnostics).unwrap();
    assert_eq!(with_diagnostics["findings"], json!([]));
    let diagnostics = with_diagnostics["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{with_diagnostics}");
    assert_eq!(diagnostics[0]["severity"], "warning");
    assert_eq!(diagnostics[0]["file"], "mutated-worker.ts");
    assert_eq!(diagnostics[0]["line"], 8);
    assert!(diagnostics[0]["message"]
        .as_str()
        .unwrap()
        .contains("worker-rate-limit-per-process was not judged"));

    let batched = super::analyze_project::analyze_project_json_impl(super::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                { "type": "queueCheck" },
                { "type": "queueCheck", "includeDiagnostics": true },
                { "type": "check" }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let batched: serde_json::Value = serde_json::from_str(&batched).unwrap();
    assert_eq!(batched["reports"][0]["result"], bare);
    assert_eq!(batched["reports"][1]["result"], with_diagnostics);
    assert_eq!(batched["reports"][2]["result"]["queues"], json!([]));
    let warnings = batched["reports"][2]["result"]["warnings"]
        .as_array()
        .unwrap();
    assert_eq!(warnings.len(), 1, "{batched}");
    assert_eq!(
        warnings[0],
        "queues-check: mutated-worker.ts:8 worker-rate-limit-per-process was not judged because the .rateLimit() receiver did not resolve"
    );
}

#[test]
fn queue_warning_suppression_matches_direct_and_batched_aggregate_checks() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/worker-rate-limit-suppression/fixture")
        .canonicalize()
        .unwrap();
    let direct = super::check_json_impl(super::options::test_json_arg(
        json!({ "root": root, "includeSuppressed": true }).to_string(),
    ))
    .unwrap();
    let direct: serde_json::Value = serde_json::from_str(&direct).unwrap();
    let warnings = direct["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 1, "{direct}");
    assert_eq!(
        warnings[0],
        format!(
            "queues-check: worker.ts:8 {}",
            crate::queue::WORKER_RATE_LIMIT_UNRESOLVED_MESSAGE
        )
    );
    assert_eq!(direct["suppressed"].as_array().unwrap().len(), 3);

    let batched = super::analyze_project::analyze_project_json_impl(super::options::test_json_arg(
        json!({
            "root": root,
            "reports": [{ "type": "check", "includeSuppressed": true }]
        })
        .to_string(),
    ))
    .unwrap();
    let batched: serde_json::Value = serde_json::from_str(&batched).unwrap();
    assert_eq!(batched["reports"][0]["result"], direct);
}
