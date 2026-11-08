use super::*;
use serde_json::{json, Value};

#[test]
fn stale_allow_entries_are_advisory_and_match_batched_check() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/unused-allow-entries");
    let report: Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root}).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(report["rules"], json!([]), "{report}");
    let advice = report["advisories"].as_array().unwrap();
    assert_eq!(advice.len(), 2, "{report}");
    assert_eq!(advice[0]["rule"], "unused-allow-entry");
    assert_eq!(advice[0]["file"], ".no-mistakes.yml");
    assert_eq!(advice[0]["line"], 13);
    assert_eq!(advice[0]["import"], "query-reached-per-item");
    assert_eq!(advice[0]["target"], "src/entry.mts:900#read");
    assert_eq!(advice[1]["target"], "src/missing.mts#retired");
    let batch: Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({"root": root, "reports": [{"type": "check"}]}).to_string(),
            ),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batch["reports"][0]["result"], report);
}

#[test]
fn unused_allow_policy_matches_batched_check() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/unused-allow-entries")
        .canonicalize()
        .unwrap();
    let mut canonical = None;
    for policy in ["advise", "report", "ignore"] {
        let config = root.join(format!("{policy}.yml"));
        let report: Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": config}).to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            2 * usize::from(policy == "report"),
            "{report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            2 * usize::from(policy == "advise"),
            "{report}"
        );
        if policy != "ignore" {
            let channel = if policy == "report" {
                "rules"
            } else {
                "advisories"
            };
            let mut finding = report[channel][0].clone();
            finding.as_object_mut().unwrap().remove("file");
            if let Some(expected) = &canonical {
                assert_eq!(&finding, expected);
            } else {
                canonical = Some(finding);
            }
        }
        let batch: Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({"root": root, "config": config, "reports": [{"type": "check"}]})
                        .to_string(),
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(batch["reports"][0]["result"], report);
    }
}

#[test]
fn unused_allow_policy_rejects_invalid_values() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/unused-allow-entries");
    let error = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({"root": root, "config": root.join("invalid.yml")}).to_string(),
    ))
    .unwrap_err()
    .to_string();
    assert!(error.contains("unsupported"), "{error}");
}

#[test]
fn report_policy_respects_narrowed_scope_and_overrides_advisory_severity() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/unused-allow-entries");
    for (config, count) in [("report-narrowed", 0), ("report-advisory", 2)] {
        let report: Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": root.join(format!("{config}.yml"))}).to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(report["rules"].as_array().unwrap().len(), count, "{report}");
        assert_eq!(report["advisories"], json!([]), "{report}");
    }
}

#[test]
fn deferred_selector_accounting_matches_batched_check() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (fixture, config_name, count) in [
        (
            "test-cases/rules/no-empty-or-comments-only-files/fixture/allow-audit",
            ".no-mistakes.yml",
            2,
        ),
        (
            "fixtures/check/postgres-pattern-exceptions",
            "report.yml",
            6,
        ),
        (
            "test-cases/rules/postgres-object-naming/fixture/ignore-patterns-stale",
            ".no-mistakes.yml",
            1,
        ),
        (
            "test-cases/rules/postgres-sql-shape-policy/fixture/ignore-tables-stale",
            ".no-mistakes.yml",
            1,
        ),
    ] {
        let root = repository.join(fixture).canonicalize().unwrap();
        let config = root.join(config_name);
        let options = crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": config}).to_string(),
        );
        let report: Value = serde_json::from_str(&check_json_impl(options).unwrap()).unwrap();
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), count, "{report}");
        assert!(findings
            .iter()
            .all(|finding| finding["rule"] == "unused-allow-entry"));
        let batch: Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({"root": root, "config": config, "reports": [{"type": "check"}]})
                        .to_string(),
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(batch["reports"][0]["result"], report);
    }
}

#[test]
fn missing_file_allowances_match_batched_check() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check/unused-allow-missing/fixture")
        .canonicalize()
        .unwrap();
    for (name, count) in [
        (".no-mistakes.yml", 15),
        ("ignore.yml", 0),
        ("narrowed.yml", 0),
        ("options-narrowed.yml", 0),
        ("windows.yml", 1),
        ("functions.yml", 1),
        ("gates.yml", 1),
        ("indices.yml", 3),
        ("hash-selectors.yml", 0),
        ("trimmed.yml", 0),
        ("storybook.yml", 2),
    ] {
        let config = root.join(name);
        let options = json!({"root": root, "config": config});
        let report: Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
        )
        .unwrap();
        assert_eq!(report["rules"].as_array().unwrap().len(), count, "{report}");
        let batch: Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({"root": root, "config": config, "reports": [{"type": "check"}]})
                        .to_string(),
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(batch["reports"][0]["result"], report);
    }
}
