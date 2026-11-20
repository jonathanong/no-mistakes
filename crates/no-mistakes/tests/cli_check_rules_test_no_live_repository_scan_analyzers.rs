const RULE: &str = "test-no-live-repository-scan";

#[test]
fn test_no_live_repository_scan_analyzer_commands_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(RULE, "analyzer-commands", "");
    let report = crate::check_json_snapshot::assert_saved_fixture_report(
        "test_no_live_repository_scan_analyzers",
        &[
            ("configured", root.clone(), root.join(".no-mistakes.yml")),
            (
                "default",
                root.clone(),
                root.join(".no-mistakes-default.yml"),
            ),
        ],
        &[],
        &[],
    );
    let configured = &report["configured"]["report"];
    let mut lines: Vec<_> = configured["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            assert_eq!(finding["target"], "analyzer subprocess");
            finding["sourceLine"].as_str().unwrap().to_string()
        })
        .collect();
    lines.sort();
    assert_eq!(
        lines,
        [
            "execFileSync(\"npx\", [\"widget-audit\", \"check\"]);",
            "execFileSync(\"pnpm\", [\"dlx\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"pnpm\", [\"exec\", \"widget-audit\", \"check\", \"--json\"]);",
            "execFileSync(\"pnpm\", [\"exec\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"widget-audit\", [\"check\"]);",
            "execFileSync(\"widget-audit\", [\"check\"], { encoding: \"utf8\" });",
            "execFileSync(widgetAudit, [\"check\"]);",
            "spawnSync(\"widget-audit\", [\"check\"], { cwd: process.cwd() });",
        ]
    );
    let mut advisories: Vec<_> = configured["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            assert_eq!(finding["target"], "unresolved root");
            finding["sourceLine"].as_str().unwrap().to_string()
        })
        .collect();
    advisories.sort();
    assert_eq!(
        advisories,
        [
            "execFileSync(\"widget-audit\", [\"check\"], options);",
            "execFileSync(\"widget-audit\", [\"check\"], { cwd: root, \"order.id\": \"1\" });",
        ]
    );
    assert!(report["default"]["report"]["rules"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(report["default"]["report"]["advisories"]
        .as_array()
        .unwrap()
        .is_empty());
}
