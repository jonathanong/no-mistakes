#[path = "common/git_tracked_rule.rs"]
mod git_tracked_rule;

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Output};

const VITEST_TIMERS: &str = "vitest-no-real-timers, application #1";
const PLAYWRIGHT_TIMERS: &str = "playwright-no-set-timeout, application #2";
const INTEGRATION_MOCKS: &str = "integration-no-mocks, application #3";
const SCRIPTS_SLEEP: &str = "scripts-no-sleep, application #4";
const YAML_APPLICATIONS: [&str; 4] = [
    VITEST_TIMERS,
    PLAYWRIGHT_TIMERS,
    INTEGRATION_MOCKS,
    SCRIPTS_SLEEP,
];

fn bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn check_json(root: &Path, config: &Path) -> (Output, Value) {
    let output = Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output, report)
}

fn rules(report: &Value) -> &[Value] {
    report["rules"].as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn has_rule(findings: &[Value], file: &str, target: &str, import: &str, application: &str) -> bool {
    has_rule_at(findings, file, target, import, application, None)
}

fn has_rule_at(
    findings: &[Value],
    file: &str,
    target: &str,
    import: &str,
    application: &str,
    line: Option<u64>,
) -> bool {
    findings.iter().any(|finding| {
        finding["file"] == file
            && finding["target"] == target
            && finding["import"] == import
            && finding["message"]
                .as_str()
                .is_some_and(|message| message.contains(application))
            && line.is_none_or(|line| finding["line"] == line)
    })
}

fn file_application(findings: &[Value], file: &str, application: &str) -> bool {
    findings.iter().any(|finding| {
        finding["file"] == file
            && finding["message"]
                .as_str()
                .is_some_and(|message| message.contains(application))
    })
}

fn application_label(finding: &Value) -> Option<&str> {
    let message = finding["message"].as_str()?;
    let start = message.find('(')? + 1;
    let end = message.find("): ")?;
    Some(&message[start..end])
}

fn yaml_application_order(findings: &[Value]) -> Vec<String> {
    let mut by_index = BTreeMap::new();
    for finding in findings {
        let Some(label) = application_label(finding) else {
            continue;
        };
        let Some(index) = label
            .rsplit_once("application #")
            .and_then(|(_, index)| index.parse::<u32>().ok())
        else {
            continue;
        };
        by_index.insert(index, label.to_string());
    }
    by_index.into_values().collect()
}

#[test]
fn consumer_matrix_reports_repeatable_timer_and_mock_policies() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (output, report) = check_json(root.path(), &root.path().join(".no-mistakes.yml"));
    let findings = rules(&report);
    assert!(!output.status.success(), "{report:#?}");

    assert_eq!(
        yaml_application_order(findings),
        YAML_APPLICATIONS,
        "{report:#?}"
    );
    assert!(
        findings.iter().all(|finding| application_label(finding)
            .is_some_and(|label| YAML_APPLICATIONS.contains(&label))),
        "{report:#?}"
    );

    assert!(has_rule_at(
        findings,
        "src/unit/timers.test.mts",
        "global `setTimeout`",
        "setTimeout",
        VITEST_TIMERS,
        Some(9),
    ));
    assert!(has_rule_at(
        findings,
        "src/unit/timers.test.mts",
        "global `setTimeout`",
        "setTimeout",
        VITEST_TIMERS,
        Some(13),
    ));
    assert!(has_rule_at(
        findings,
        "src/unit/timers.test.mts",
        "module export `node:timers#setTimeout`",
        "wait",
        VITEST_TIMERS,
        Some(14),
    ));
    assert!(has_rule_at(
        findings,
        "src/unit/timers.test.mts",
        "module export `node:timers#setTimeout`",
        "timers.setTimeout",
        VITEST_TIMERS,
        Some(15),
    ));
    assert!(has_rule_at(
        findings,
        "src/unit/timers.test.mts",
        "module export `node:timers/promises#setTimeout`",
        "waitP",
        VITEST_TIMERS,
        Some(16),
    ));
    assert!(has_rule_at(
        findings,
        "src/unit/timers.test.mts",
        "module export `node:timers/promises#setTimeout`",
        "timersP.setTimeout",
        VITEST_TIMERS,
        Some(17),
    ));
    assert!(has_rule_at(
        findings,
        "src/integration/timers.test.mts",
        "global `setTimeout`",
        "setTimeout",
        VITEST_TIMERS,
        Some(2),
    ));

    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "global `setTimeout`",
        "setTimeout",
        PLAYWRIGHT_TIMERS,
        Some(7),
    ));
    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "module export `node:timers#setTimeout`",
        "wait",
        PLAYWRIGHT_TIMERS,
        Some(8),
    ));
    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "module export `node:timers#setTimeout`",
        "timers.setTimeout",
        PLAYWRIGHT_TIMERS,
        Some(9),
    ));
    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "module export `node:timers/promises#setTimeout`",
        "waitP",
        PLAYWRIGHT_TIMERS,
        Some(10),
    ));
    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "module export `node:timers/promises#setTimeout`",
        "timersP.setTimeout",
        PLAYWRIGHT_TIMERS,
        Some(11),
    ));
    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "terminal `waitForTimeout`",
        "page.waitForTimeout",
        PLAYWRIGHT_TIMERS,
        Some(33),
    ));
    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "terminal `waitForTimeout`",
        "browser.waitForTimeout",
        PLAYWRIGHT_TIMERS,
        Some(34),
    ));
    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "terminal `waitForTimeout`",
        "page.waitForTimeout",
        PLAYWRIGHT_TIMERS,
        Some(36),
    ));
    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "terminal `waitForTimeout`",
        "<unknown>.waitForTimeout",
        PLAYWRIGHT_TIMERS,
        Some(37),
    ));
    assert!(has_rule_at(
        findings,
        "e2e/timers.spec.ts",
        "terminal `waitForTimeout`",
        "this.waitForTimeout",
        PLAYWRIGHT_TIMERS,
        Some(17),
    ));
    assert!(has_rule_at(
        findings,
        "scripts/sleep.mts",
        "global `setTimeout`",
        "setTimeout",
        SCRIPTS_SLEEP,
        Some(2),
    ));

    for (target, line) in [
        ("vi.mock", 14),
        ("vi.doMock", 15),
        ("vi.importMock", 16),
        ("vi.fn", 17),
        ("vi.spyOn", 18),
        ("vi.stubGlobal", 19),
        ("jest.mock", 20),
        ("jest.doMock", 21),
    ] {
        assert!(
            has_rule_at(
                findings,
                "src/integration/mocks.test.mts",
                &format!("exact `{target}`"),
                target,
                INTEGRATION_MOCKS,
                Some(line),
            ),
            "{target}: {report:#?}"
        );
    }

    let unit_mocks = std::fs::read_to_string(root.path().join("src/unit/mocks.test.mts")).unwrap();
    assert!(unit_mocks.contains("vi.mock"), "{unit_mocks}");
    assert!(unit_mocks.contains("vi.fn"), "{unit_mocks}");
    assert!(has_rule_at(
        findings,
        "src/unit/mocks.test.mts",
        "module export `node:timers#setTimeout`",
        "wait",
        VITEST_TIMERS,
        Some(11),
    ));
    assert!(!file_application(
        findings,
        "src/unit/mocks.test.mts",
        "integration-no-mocks"
    ));

    let local_timeout =
        std::fs::read_to_string(root.path().join("src/unit/local-timeout.test.mts")).unwrap();
    assert!(
        local_timeout.contains("import { setTimeout } from \"../clock.mts\""),
        "{local_timeout}"
    );
    assert!(has_rule_at(
        findings,
        "src/unit/local-timeout.test.mts",
        "module export `node:timers#setTimeout`",
        "wait",
        VITEST_TIMERS,
        Some(7),
    ));
    assert!(
        !findings.iter().any(|finding| {
            finding["file"] == "src/unit/local-timeout.test.mts"
                && finding["import"] == "setTimeout"
                && (finding["target"] == "global `setTimeout`"
                    || finding["target"] == "module export `node:timers#setTimeout`"
                    || finding["target"] == "module export `node:timers/promises#setTimeout`")
        }),
        "{report:#?}"
    );

    assert!(has_rule_at(
        findings,
        "src/unit/allowed.test.mts",
        "module export `node:timers#setTimeout`",
        "wait",
        VITEST_TIMERS,
        Some(5),
    ));
    assert!(!has_rule(
        findings,
        "src/unit/allowed.test.mts",
        "global `setTimeout`",
        "setTimeout",
        VITEST_TIMERS,
    ));
    assert!(!has_rule(
        findings,
        "src/unit/allowed.test.mts",
        "global `setTimeout`",
        "test.setTimeout",
        VITEST_TIMERS,
    ));

    assert!(has_rule_at(
        findings,
        "src/unit/suppressed.test.mts",
        "module export `node:timers#setTimeout`",
        "wait",
        VITEST_TIMERS,
        Some(6),
    ));
    assert!(!has_rule(
        findings,
        "src/unit/suppressed.test.mts",
        "global `setTimeout`",
        "setTimeout",
        VITEST_TIMERS,
    ));

    assert!(has_rule_at(
        findings,
        "src/unit/dynamic.test.mts",
        "module export `node:timers#setTimeout`",
        "wait",
        VITEST_TIMERS,
        Some(3),
    ));
    assert!(!findings.iter().any(|finding| {
        finding["file"] == "src/unit/dynamic.test.mts" && finding["target"] == "unknown call"
    }));

    let same_file_globals = findings
        .iter()
        .filter(|finding| {
            finding["file"] == "src/unit/timers.test.mts"
                && finding["target"] == "global `setTimeout`"
        })
        .count();
    assert_eq!(same_file_globals, 2, "{report:#?}");

    assert!(!file_application(
        findings,
        "src/app.mts",
        "vitest-no-real-timers"
    ));
    assert!(!file_application(findings, "src/helper.mts", "application"));
    assert!(!file_application(
        findings,
        "e2e/timers.spec.ts",
        "vitest-no-real-timers"
    ));
    assert!(!file_application(
        findings,
        "src/unit/timers.test.mts",
        "playwright-no-set-timeout"
    ));
    assert!(!file_application(
        findings,
        "src/unit/timers.test.mts",
        "integration-no-mocks"
    ));
    assert!(!findings.iter().any(|finding| {
        finding["file"] == "e2e/timers.spec.ts" && finding["import"] == "<unknown>"
    }));
}
