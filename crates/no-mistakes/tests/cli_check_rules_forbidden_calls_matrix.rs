#[path = "common/git_tracked_rule.rs"]
mod git_tracked_rule;

use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

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
    findings.iter().any(|finding| {
        finding["file"] == file
            && finding["target"] == target
            && finding["import"] == import
            && finding["message"]
                .as_str()
                .is_some_and(|message| message.contains(application))
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

#[test]
fn consumer_matrix_reports_repeatable_timer_and_mock_policies() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "consumer-matrix");
    let (output, report) = check_json(root.path(), &root.path().join(".no-mistakes.yml"));
    let findings = rules(&report);
    assert!(!output.status.success(), "{report:#?}");

    assert!(has_rule(
        findings,
        "src/unit/timers.test.mts",
        "global `setTimeout`",
        "setTimeout",
        "vitest-no-real-timers, application #1",
    ));
    assert!(has_rule(
        findings,
        "src/unit/timers.test.mts",
        "module export `node:timers#setTimeout`",
        "wait",
        "vitest-no-real-timers, application #1",
    ));
    assert!(has_rule(
        findings,
        "src/unit/timers.test.mts",
        "module export `node:timers#setTimeout`",
        "timers.setTimeout",
        "vitest-no-real-timers, application #1",
    ));
    assert!(has_rule(
        findings,
        "src/unit/timers.test.mts",
        "module export `node:timers/promises#setTimeout`",
        "waitP",
        "vitest-no-real-timers, application #1",
    ));
    assert!(has_rule(
        findings,
        "src/unit/timers.test.mts",
        "module export `node:timers/promises#setTimeout`",
        "timersP.setTimeout",
        "vitest-no-real-timers, application #1",
    ));
    assert!(has_rule(
        findings,
        "src/integration/timers.test.mts",
        "global `setTimeout`",
        "setTimeout",
        "vitest-no-real-timers, application #1",
    ));

    assert!(has_rule(
        findings,
        "e2e/timers.spec.ts",
        "global `setTimeout`",
        "setTimeout",
        "playwright-no-set-timeout, application #2",
    ));
    assert!(has_rule(
        findings,
        "e2e/timers.spec.ts",
        "module export `node:timers#setTimeout`",
        "wait",
        "playwright-no-set-timeout, application #2",
    ));
    assert!(has_rule(
        findings,
        "e2e/timers.spec.ts",
        "terminal `waitForTimeout`",
        "page.waitForTimeout",
        "playwright-no-set-timeout, application #2",
    ));
    assert!(has_rule(
        findings,
        "e2e/timers.spec.ts",
        "terminal `waitForTimeout`",
        "browser.waitForTimeout",
        "playwright-no-set-timeout, application #2",
    ));
    assert!(has_rule(
        findings,
        "e2e/timers.spec.ts",
        "terminal `waitForTimeout`",
        "<unknown>.waitForTimeout",
        "playwright-no-set-timeout, application #2",
    ));
    assert!(has_rule(
        findings,
        "e2e/timers.spec.ts",
        "terminal `waitForTimeout`",
        "this.waitForTimeout",
        "playwright-no-set-timeout, application #2",
    ));
    assert!(has_rule(
        findings,
        "scripts/sleep.mts",
        "global `setTimeout`",
        "setTimeout",
        "scripts-no-sleep, application #4",
    ));

    for target in [
        "vi.mock",
        "vi.doMock",
        "vi.importMock",
        "vi.fn",
        "vi.spyOn",
        "vi.stubGlobal",
        "jest.mock",
        "jest.doMock",
    ] {
        assert!(
            has_rule(
                findings,
                "src/integration/mocks.test.mts",
                &format!("exact `{target}`"),
                target,
                "integration-no-mocks, application #3",
            ),
            "{target}: {report:#?}"
        );
    }

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
        "src/unit/allowed.test.mts",
        "application"
    ));
    assert!(!file_application(
        findings,
        "src/unit/suppressed.test.mts",
        "application"
    ));
    assert!(!file_application(
        findings,
        "src/unit/dynamic.test.mts",
        "application"
    ));
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
