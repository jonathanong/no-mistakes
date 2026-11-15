use std::path::PathBuf;

const RULE: &str = "test-no-live-repository-scan";
const CASES: [&str; 9] = [
    "helper-owned",
    "typed-call",
    "it-owned",
    "object-property",
    "helper-live",
    "exported-live",
    "argv-flag",
    "stdin-json",
    "omitted-argument",
];

#[test]
fn issue_3355_parameter_roots_check_json_snapshot() {
    let cases: Vec<(&str, PathBuf, PathBuf)> = CASES
        .into_iter()
        .map(|name| {
            let root = crate::check_json_snapshot::rules_path(RULE, "issue-3355", name);
            (name, root.clone(), root.join(".no-mistakes.yml"))
        })
        .collect();
    let report = crate::check_json_snapshot::assert_saved_fixture_report(
        "test_no_live_repository_scan_issue_3355",
        &cases,
        &[],
        &[],
    );
    for name in ["helper-owned", "typed-call", "it-owned", "object-property"] {
        let body = &report[name]["report"];
        assert!(body["rules"].as_array().unwrap().is_empty(), "{name}");
        assert!(body["advisories"].as_array().unwrap().is_empty(), "{name}");
    }
    for name in [
        "helper-live",
        "exported-live",
        "argv-flag",
        "stdin-json",
        "omitted-argument",
    ] {
        assert_eq!(
            report[name]["report"]["advisories"]
                .as_array()
                .unwrap()
                .len(),
            1,
            "{name}"
        );
    }
}
