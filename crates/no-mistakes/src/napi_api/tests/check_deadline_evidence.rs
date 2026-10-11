use super::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/integration-tests/declared-deadlines/fixture")
}

fn report(config: Option<&str>, evidence: bool, suppressed: bool) -> serde_json::Value {
    serde_json::from_str(
        &check_json_impl(json!({
            "root": root(), "config": config,
            "includeRunnerConfigDeadlines": evidence, "includeSuppressed": suppressed,
        }))
        .unwrap(),
    )
    .unwrap()
}

fn projects(value: &serde_json::Value, framework: &str) -> Vec<serde_json::Value> {
    value["runnerConfigDeadlines"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|runner| runner["framework"] == framework)
        .flat_map(|runner| runner["configs"].as_array().unwrap())
        .flat_map(|config| config["projects"].as_array().unwrap())
        .cloned()
        .collect()
}

#[test]
fn public_deadline_evidence_preserves_invalid_unknown_absent_and_provenance() {
    let baseline = report(None, false, false);
    let evidence = report(None, true, false);
    let mut comparable = evidence.clone();
    comparable
        .as_object_mut()
        .unwrap()
        .remove("runnerConfigDeadlines");
    assert_eq!(baseline, comparable);
    assert!(baseline.get("runnerConfigDeadlines").is_none());
    assert!(evidence["integration"].as_array().unwrap().is_empty());
    let vitest = projects(&evidence, "vitest");
    let negative = vitest
        .iter()
        .find(|project| project["case"]["milliseconds"].as_f64() == Some(-1.0))
        .unwrap();
    assert_eq!(negative["case"]["status"], "known");
    assert_eq!(negative["case"]["provenance"]["path"], "vitest.negative.ts");
    assert!(negative["case"]["provenance"]["span"].is_array());
    let duplicates = vitest
        .iter()
        .filter(|project| project["policyName"] == "negative")
        .collect::<Vec<_>>();
    assert_eq!(duplicates.len(), 2);
    assert_ne!(duplicates[0]["config"], duplicates[1]["config"]);
    let invalid = vitest
        .iter()
        .find(|project| project["policyName"] == "invalid-literal")
        .unwrap();
    assert_eq!(invalid["case"]["milliseconds"].as_f64(), Some(0.0));
    assert_eq!(invalid["hook"]["milliseconds"].as_f64(), Some(30001.0));
    assert_eq!(invalid["fixture"]["status"], "absent");
    let absent = vitest
        .iter()
        .find(|project| {
            project["config"] == "vitest.projects.json" && project["case"]["status"] == "absent"
        })
        .unwrap();
    assert_eq!(absent["hook"]["status"], "absent");
    let json_known = vitest
        .iter()
        .find(|project| {
            project["config"] == "vitest.projects.json" && project["case"]["status"] == "known"
        })
        .unwrap();
    assert!(json_known["case"]["provenance"]["span"].is_null());
    let local = vitest
        .iter()
        .find(|project| project["config"] == "vitest.root-opaque.ts")
        .unwrap();
    assert_eq!(local["case"]["milliseconds"].as_f64(), Some(1.0));
    assert_eq!(local["hook"]["status"], "unknown");
    let merge = vitest
        .iter()
        .find(|project| project["config"] == "vitest.merge-direct.ts")
        .unwrap();
    assert_eq!(merge["case"]["milliseconds"].as_f64(), Some(30001.0));
    assert_eq!(merge["hook"]["milliseconds"].as_f64(), Some(1.0));
    assert!(vitest
        .iter()
        .any(|project| project["case"]["reason"] == "accessor"));
    assert!(vitest
        .iter()
        .any(|project| project["case"]["reason"] == "unprovedBinding"));
    assert!(vitest
        .iter()
        .any(|project| project["case"]["reason"] == "unsupportedConfigCall"));
    let playwright = projects(&evidence, "playwright");
    let inherited = playwright
        .iter()
        .find(|project| project["policyName"] == "inherited")
        .unwrap();
    assert_eq!(inherited["case"]["milliseconds"].as_f64(), Some(30000.0));
    assert!(!inherited["case"]["provenance"]["inheritedThrough"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(inherited["hook"]["status"], "absent");
    assert_eq!(inherited["fixture"]["status"], "absent");
    assert!(playwright
        .iter()
        .any(|project| project["case"]["reason"] == "opaqueSpread"));
}

#[test]
fn public_deadline_reports_keep_not_requested_empty_and_failures_explicit() {
    let none = report(Some("none.yml"), true, false);
    assert!(none["runnerConfigDeadlines"]
        .as_array()
        .unwrap()
        .iter()
        .all(|runner| runner["status"] == "notRequested"));
    let empty = report(Some("empty.yml"), true, false);
    let vitest = empty["runnerConfigDeadlines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|runner| runner["framework"] == "vitest")
        .unwrap();
    assert_eq!(vitest["status"], "prepared");
    assert!(vitest["configs"].as_array().unwrap().is_empty());
    let failures = report(Some("failures.yml"), true, false);
    let failed = failures["runnerConfigDeadlines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|runner| runner["framework"] == "vitest")
        .unwrap();
    assert_eq!(failed["status"], "failed");
    assert_eq!(failed["configs"].as_array().unwrap().len(), 2);
    assert!(failed["configs"]
        .as_array()
        .unwrap()
        .iter()
        .all(
            |config| config["status"] == "failed" && !config["error"].as_str().unwrap().is_empty()
        ));
}

#[test]
fn public_check_cli_renderer_and_batched_reports_share_evidence_without_ordinary_changes() {
    let direct = report(None, true, false);
    let root = root();
    let results =
        crate::check_runner::run_all_with_evidence(root.clone(), None, None, false, true).unwrap();
    assert_eq!(crate::check_runner::json_value(&results), direct);
    let batch: serde_json::Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(json!({ "root": root, "reports": [
            { "type": "check", "id": "evidence", "includeRunnerConfigDeadlines": true },
            { "type": "check", "id": "ordinary" },
            { "type": "check", "id": "suppressed", "includeRunnerConfigDeadlines": true, "includeSuppressed": true }
        ] })).unwrap()).unwrap();
    assert_eq!(batch["reports"][0]["result"], direct);
    assert_eq!(batch["reports"][1]["result"], report(None, false, false));
    let mut audit = batch["reports"][2]["result"].clone();
    assert!(audit.get("suppressed").is_some());
    audit.as_object_mut().unwrap().remove("suppressed");
    assert_eq!(audit, direct);
}

#[test]
fn ordinary_integration_errors_are_preserved_when_deadline_evidence_is_requested() {
    for config in ["integration-error.yml", "integration-missing.yml"] {
        let baseline = check_json_impl(json!({"root": root(), "config": config})).unwrap_err();
        let evidence = check_json_impl(
            json!({"root": root(), "config": config, "includeRunnerConfigDeadlines": true}),
        )
        .unwrap_err();
        assert_eq!(baseline.reason, evidence.reason);
        assert!(!baseline.reason.is_empty());
    }
}
