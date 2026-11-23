use super::{parsed, policy, RULE_ID};

const OPTIONS: &str = r#"
actions:
  - uses: example/publish-widget@v1
    allowedInputs: [name]
  - uses: example/publish-widget
    allowedInputs: [" ", path]
    requiredPresent: [name, name]
"#;

const STEP: &str = "jobs:\n  build:\n    steps:\n      - uses: example/publish-widget@v2\n";

#[test]
fn allowlists_and_required_present_merge_per_family() {
    let silent = format!("{STEP}        with:\n          name: a\n          path: b\n");
    assert!(parsed(&silent, OPTIONS).is_empty());
    let extra = format!("{STEP}        with:\n          name: a\n          other: b\n");
    let findings = parsed(&extra, OPTIONS);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("`other`"));
    let missing = format!("{STEP}        with:\n          name:\n");
    assert_eq!(parsed(&missing, OPTIONS).len(), 1);
    assert_eq!(parsed(STEP, OPTIONS).len(), 1);
}

#[test]
fn banned_inputs_report_once_without_an_allowlist_finding() {
    let options =
        "actions:\n  - uses: example/publish-widget\n    allowedInputs: [name]\n    ban: [other]\n";
    let source = format!("{STEP}        with:\n          other: b\n");
    let findings = parsed(&source, options);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("banned"));
}

#[test]
fn config_rejects_allowlist_contradictions() {
    let cases = [
        "actions:\n  - uses: a/b\n    allowedInputs: [x]\n    ban: [x]\n",
        "actions:\n  - uses: a/b\n    allowedInputs: [x]\n    require:\n      y: 1\n",
        "actions:\n  - uses: a/b\n    allowedInputs: [x]\n    requiredPresent: [y]\n",
        "actions:\n  - uses: a/b\n    requiredPresent: [y]\n    ban: [y]\n",
        "actions:\n  - uses: a/b\n    requiredPresent: [y]\n    require:\n      y: 1\n",
    ];
    for yaml in cases {
        let error = policy::compile(serde_yaml::from_str(yaml).unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains(RULE_ID), "{error}");
    }
}
