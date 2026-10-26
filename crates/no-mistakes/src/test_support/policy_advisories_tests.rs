use super::*;

#[test]
fn policy_view_preserves_unrelated_advisories_in_nested_reports() {
    let report = serde_json::json!({
        "advisories": [
            { "rule": "unrelated-existing-rule" },
            { "rule": "unused-disable-directive" },
            { "rule": "invalid-disable-directive" }
        ],
        "reports": [{ "result": { "advisories": [{ "rule": "another-existing-rule" }] } }]
    });
    let view = without_hygiene_advisory_json(report);
    assert_eq!(
        view["advisories"],
        serde_json::json!([{ "rule": "unrelated-existing-rule" }])
    );
    assert_eq!(
        view["reports"][0]["result"]["advisories"],
        serde_json::json!([{ "rule": "another-existing-rule" }])
    );
}
