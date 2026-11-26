use super::*;
use crate::napi_api::options::test_json_arg;
use serde_json::Value;

#[test]
fn check_preserves_workflow_findings_when_yaml_merge_is_unlocated() {
    let fixture = crate::test_support::rule_fixture_root("workflow-topology-policy");
    let root = fixture.join("merge-location-scoped");
    for (mode, advisory_count) in [("advise", 1), ("ignore", 0)] {
        let config = fixture
            .parent()
            .unwrap()
            .join(format!("configs/merge-location-scoped-{mode}.yml"));
        let input = test_json_arg(json!({ "root": &root, "config": config }).to_string());
        let report: Value = serde_json::from_str(&check_json_impl(input).unwrap()).unwrap();
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 8, "{report}");
        assert!(
            findings.iter().any(|row| {
                row["message"]
                    .as_str()
                    .is_some_and(|message| message.starts_with("forbidden workflow event"))
            }),
            "{report}"
        );
        let advisories = report["advisories"].as_array().unwrap();
        assert_eq!(advisories.len(), advisory_count, "{report}");
        if let Some(advisory) = advisories.first() {
            assert_eq!(advisory["target"], "unanalyzable");
        }
    }
}
