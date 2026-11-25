use super::*;
use crate::napi_api::options::test_json_arg;
use serde_json::Value;

#[test]
fn check_insta_snapshot_updates_matches_the_batched_report() {
    let fixture =
        crate::test_support::rule_fixture_root("github-actions-no-insta-snapshot-updates");
    let root = fixture.join("suppression");
    let config = fixture.parent().unwrap().join("configs/enabled.yml");
    let report: Value = serde_json::from_str(
        &check_json_impl(test_json_arg(
            json!({ "root": root, "config": config, "includeSuppressed": true }).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    let batch: Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(test_json_arg(
            json!({ "root": root, "reports": [{ "type": "check", "config": config, "includeSuppressed": true }] }).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batch["reports"][0]["result"], report);
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(
        findings[0]["rule"],
        crate::codebase::rules::GITHUB_ACTIONS_NO_INSTA_SNAPSHOT_UPDATES
    );
    assert_eq!(findings[0]["line"], 16, "{report}");
    let kinds: std::collections::BTreeSet<_> = report["suppressed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["directive"]["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["file", "line", "nextLine"].into(), "{report}");
}
