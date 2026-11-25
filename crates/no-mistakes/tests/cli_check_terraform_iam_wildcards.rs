use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "terraform-iam-no-undocumented-wildcards";

#[test]
fn terraform_iam_wildcard_reports_match_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    let reports = assert_saved_fixture_report(
        "terraform_iam_wildcard_reports",
        &[
            ("configured", root.clone(), root.join(".no-mistakes.yml")),
            ("advise", root.clone(), root.join("advise.yml")),
            ("ignore", root.clone(), root.join("ignore.yml")),
            ("default_off", root.clone(), root.join("default.yml")),
            ("incomplete", root.clone(), root.join("incomplete.yml")),
        ],
        &[],
        &["--include-suppressed"],
    );

    let configured = &reports["configured"];
    assert_eq!(configured["exitCode"], 1);
    assert!(configured["report"].get("incomplete").is_none());
    let mut configured_locations =
        finding_locations(configured["report"]["rules"].as_array().unwrap());
    configured_locations.sort_unstable();
    assert_eq!(
        configured_locations,
        [
            (Some("policies/accepted.tf"), Some(12), Some("unanalyzable")),
            (Some("policies/accepted.tf"), Some(18), Some("unanalyzable")),
            (Some("policies/accepted.tf"), Some(22), Some("unanalyzable")),
            (Some("policies/accepted.tf"), Some(32), Some("unanalyzable")),
            (Some("policies/accepted.tf"), Some(62), Some("unanalyzable")),
            (Some("policies/accepted.tf"), Some(75), Some("unanalyzable")),
            (Some("policies/accepted.tf"), Some(79), Some("unanalyzable")),
            (
                Some("policies/example.tf.json"),
                Some(1),
                Some("unanalyzable")
            ),
            (Some("policies/receipt.tf"), Some(4), None),
            (Some("policies/receipt.tf"), Some(10), None),
            (Some("policies/receipt.tf"), Some(19), None),
            (Some("policies/receipt.tf"), Some(24), None),
            (Some("policies/receipt.tf"), Some(29), None),
        ]
    );
    assert_eq!(configured["report"]["advisories"], serde_json::json!([]));
    assert_eq!(
        configured["report"]["suppressed"].as_array().unwrap().len(),
        3
    );

    let advise = &reports["advise"];
    assert_eq!(advise["exitCode"], 1);
    let mut advise_rules = finding_locations(advise["report"]["rules"].as_array().unwrap());
    advise_rules.sort_unstable();
    assert_eq!(
        advise_rules,
        [
            (Some("policies/receipt.tf"), Some(4), None),
            (Some("policies/receipt.tf"), Some(10), None),
            (Some("policies/receipt.tf"), Some(19), None),
            (Some("policies/receipt.tf"), Some(24), None),
            (Some("policies/receipt.tf"), Some(29), None),
        ]
    );
    assert_eq!(advise["report"]["advisories"].as_array().unwrap().len(), 8);

    let ignore = &reports["ignore"];
    assert_eq!(ignore["exitCode"], 1);
    let mut ignore_rules = finding_locations(ignore["report"]["rules"].as_array().unwrap());
    ignore_rules.sort_unstable();
    assert_eq!(ignore_rules, advise_rules);
    assert_eq!(ignore["report"]["advisories"], serde_json::json!([]));

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

fn finding_locations(
    findings: &[serde_json::Value],
) -> Vec<(Option<&str>, Option<u64>, Option<&str>)> {
    findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            (
                finding["file"].as_str(),
                finding["line"].as_u64(),
                finding["target"].as_str(),
            )
        })
        .collect()
}
