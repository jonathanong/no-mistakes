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
            "exec(\"pnpm exec widget-audit check\");",
            "execFileSync(\"bunx\", [\"widget-audit\", \"check\"]);",
            "execFileSync(\"npm\", [\"exec\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"npx\", [\"--no-install\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"npx\", [\"-p\", \"widget-tools\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"npx\", [\"pnpm\", \"-C\", \"dir\", \"exec\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"npx\", [\"pnpm\", \"exec\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"npx\", [\"widget-audit\", \"check\"]);",
            "execFileSync(\"pnpm\", [\"--filter\", \"widgets\", \"exec\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"pnpm\", [\"--reporter\", \"silent\", \"exec\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"pnpm\", [\"-C\", \"dir\", \"exec\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"pnpm\", [\"-r\", \"exec\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"pnpm\", [\"dlx\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"pnpm\", [\"exec\", \"widget-audit\", \"check\", \"--json\"]);",
            "execFileSync(\"pnpm\", [\"exec\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"widget-audit\", [\"check\"]);",
            "execFileSync(\"widget-audit\", [\"check\"], { encoding: \"utf8\" });",
            "execFileSync(\"yarn\", [\"--cwd\", \"widgets\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"yarn\", [\"dlx\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"yarn\", [\"run\", \"widget-audit\", \"check\"]);",
            "execFileSync(\"yarn\", [\"widget-audit\", \"check\"]);",
            "execFileSync(widgetAudit, [\"check\"]);",
            "execSync(\"ITEM=1 widget-audit check\");",
            "execSync(\"cd widgets && ITEM=1 pnpm exec widget-audit check\");",
            "execSync(\"cd widgets && widget-audit check\");",
            "execSync(\"pnpm exec widget-audit check\");",
            "execSync(\"pnpm exec widget-audit check\", { encoding: \"utf8\" });",
            "execSync(widgetCommand);",
            "fork(\"./node_modules/.bin/widget-audit\", [\"check\"]);",
            "fork(\"./widget-audit.cjs\", [\"check\"]);",
            "fork(\"./widget-audit.js\", [\"check\"]);",
            "fork(\"./widgets/widget-audit.mjs\", [\"check\"]);",
            "spawn(\"pnpm exec widget-audit check\", { shell: true });",
            "spawn(\"pnpm\", [\"exec\", \"widget-audit\", \"check\"], { shell: true });",
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

#[test]
fn test_no_live_repository_scan_analyzer_suppression_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(RULE, "analyzer-suppress", "");
    let report = crate::check_json_snapshot::assert_saved_fixture_report(
        "test_no_live_repository_scan_analyzer_suppress",
        &[("configured", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let configured = &report["configured"]["report"];
    let lines: Vec<_> = configured["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            assert_eq!(finding["target"], "analyzer subprocess");
            finding["sourceLine"].as_str().unwrap().to_string()
        })
        .collect();
    assert_eq!(lines, ["execSync(\"pnpm exec widget-audit check\");"]);
    assert!(configured["advisories"].as_array().unwrap().is_empty());
}
