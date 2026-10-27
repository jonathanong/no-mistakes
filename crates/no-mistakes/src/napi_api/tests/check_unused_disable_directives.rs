use super::*;
use serde_json::{json, Value};

#[test]
fn host_language_comments_and_fixture_exclusions_match_batched_check() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/suppression-comment-boundaries");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root }).to_string(),
    ))
    .unwrap();
    let direct: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(direct["rules"], json!([]), "{direct}");
    let advice = direct["advisories"].as_array().unwrap();
    assert_eq!(advice.len(), 2, "{direct}");
    assert!(advice
        .iter()
        .all(|item| item["rule"] == "unused-disable-directive"));
    let output = crate::napi_api::analyze_project::analyze_project_json_impl(
        crate::napi_api::options::test_json_arg(
            json!({ "root": root, "reports": [{ "type": "check" }] }).to_string(),
        ),
    )
    .unwrap();
    let batch: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(batch["reports"][0]["result"], direct);
}

fn report(mode: Option<&str>, include_suppressed: bool) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/unused-disable-directives");
    let mut options = json!({ "root": root, "includeSuppressed": include_suppressed });
    if let Some(mode) = mode {
        options["config"] = json!(root.join(format!("{mode}.yml")));
    }
    serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap()
}

#[test]
fn hygiene_defaults_to_advisory_and_accounts_exact_consumed_directives() {
    let value = report(None, false);
    assert_eq!(value["rules"], json!([]), "{value}");
    assert!(value.get("suppressed").is_none());
    let findings = value["advisories"].as_array().unwrap();
    let mut actual: Vec<_> = findings
        .iter()
        .map(|item| {
            (
                item["file"].as_str().unwrap(),
                item["line"].as_u64().unwrap(),
                item["rule"].as_str().unwrap(),
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        vec![
            ("web/app/stale.ts", 1, "unused-disable-directive"),
            ("web/app/stale.ts", 2, "unused-disable-directive"),
            ("web/app/stale.ts", 4, "unused-disable-directive"),
            ("web/app/stale.ts", 5, "invalid-disable-directive"),
            ("web/app/stale.ts", 7, "invalid-disable-directive"),
            ("web/app/stale.ts", 9, "invalid-disable-directive"),
        ]
    );
    assert!(findings
        .iter()
        .any(|item| item["line"] == 5 && item["message"].as_str().unwrap().contains("disabled")));
    assert!(findings
        .iter()
        .any(|item| item["line"] == 7 && item["message"].as_str().unwrap().contains("unknown")));
}

#[test]
fn hygiene_report_fails_while_ignore_preserves_suppression() {
    let reported = report(Some("report"), false);
    assert_eq!(reported["rules"].as_array().unwrap().len(), 6, "{reported}");
    assert_eq!(reported["advisories"], json!([]));
    let ignored = report(Some("ignore"), true);
    assert_eq!(ignored["rules"], json!([]));
    assert_eq!(ignored["advisories"], json!([]));
    assert_eq!(
        ignored["suppressed"].as_array().unwrap().len(),
        3,
        "{ignored}"
    );
}

#[test]
fn hygiene_is_independent_of_optional_suppressed_accounting() {
    let baseline = report(None, false);
    let audit = report(None, true);
    assert_eq!(baseline["rules"], audit["rules"]);
    assert_eq!(baseline["advisories"], audit["advisories"]);
    assert_eq!(audit["suppressed"].as_array().unwrap().len(), 3);
}

#[test]
fn batched_project_check_matches_direct_check_hygiene() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/unused-disable-directives");
    let output = crate::napi_api::analyze_project::analyze_project_json_impl(
        crate::napi_api::options::test_json_arg(
            json!({
                "root": root, "reports": [{ "type": "check", "id": "audit" }]
            })
            .to_string(),
        ),
    )
    .unwrap();
    let project: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        project["reports"][0]["result"]["advisories"],
        report(None, false)["advisories"]
    );
}
