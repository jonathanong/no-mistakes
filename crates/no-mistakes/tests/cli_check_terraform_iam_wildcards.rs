use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "terraform-iam-no-undocumented-wildcards";

#[test]
fn terraform_iam_wildcard_reports_match_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    let reports = assert_saved_fixture_report(
        "terraform_iam_wildcard_reports",
        &[
            ("configured", root.clone(), root.join(".no-mistakes.yml")),
            ("default_off", root.clone(), root.join("default.yml")),
            ("incomplete", root.clone(), root.join("incomplete.yml")),
        ],
        &[],
        &["--include-suppressed"],
    );

    let configured = &reports["configured"];
    assert_eq!(configured["exitCode"], 1);
    assert!(configured["report"].get("incomplete").is_none());
    let findings = configured["report"]["rules"].as_array().unwrap();
    let mut locations: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            (finding["file"].as_str(), finding["line"].as_u64())
        })
        .collect();
    locations.sort_unstable();
    assert_eq!(
        locations,
        [
            (Some("policies/receipt.tf"), Some(4)),
            (Some("policies/receipt.tf"), Some(10))
        ]
    );
    assert_eq!(
        configured["report"]["suppressed"].as_array().unwrap().len(),
        3
    );

    let default = &reports["default_off"];
    assert_eq!(default["exitCode"], 0);
    assert_eq!(default["report"]["rules"], serde_json::json!([]));

    let incomplete = &reports["incomplete"];
    assert_eq!(incomplete["exitCode"], 1);
    let failures = incomplete["report"]["incomplete"].as_array().unwrap();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0]["consumer"], RULE);
    assert_eq!(failures[0]["file"], "incomplete/broken.tf");
    let findings = incomplete["report"]["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["file"], "incomplete/receipt.tf");
    assert_eq!(findings[0]["line"], 2);
}
