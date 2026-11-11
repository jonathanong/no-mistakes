use crate::saved_fixture;
use std::path::PathBuf;
use std::process::Command;
fn fixture() -> tempfile::TempDir {
    saved_fixture::materialize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/coverage-producer-assertion-owner/fixture"),
    )
}
#[test]
fn cli_projects_explicit_assertion_ownership() {
    let fixture = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "dependencies",
            "src/producers.mts#crossProcess",
            "--relationship",
            "assertion-owner",
            "--symbols",
            "--format",
            "json",
            "--root",
        ])
        .arg(fixture.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value["roots"],
        serde_json::json!(["src/producers.mts#crossProcess"])
    );
    assert!(
        value["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["file"] == "src/assertions.mts"
                && node["symbol"] == "assertResult"
                && node["depth"] == 1
                && node["via"] == serde_json::json!(["assertion-owner"])),
        "{value}"
    );
}
#[test]
fn cli_check_exposes_coverage_producer_findings() {
    let fixture = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(fixture.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = value["rules"].as_array().unwrap();
    assert!(findings.iter().any(
        |finding| finding["rule"] == "coverage-producer-assertion-owner"
            && finding["target"] == "productionOnly"
    ));
    for accepted in [
        "direct",
        "imported",
        "callback",
        "inlineCallback",
        "crossProcess",
        "assertionCtor",
    ] {
        assert!(
            !findings.iter().any(|finding| finding["target"] == accepted),
            "{value}"
        );
    }
}

#[test]
fn cli_reports_generator_uncertainty_without_assertion_proof() {
    let fixture = fixture();
    for (policy, findings, advisories) in [("report", 7, 0), ("advise", 0, 7), ("ignore", 0, 0)] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(fixture.path())
            .arg("--config")
            .arg(fixture.path().join(format!("generators-{policy}.yml")))
            .output()
            .unwrap();
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{report}"
        );
        assert_eq!(
            super::policy_advisories::without_hygiene(&report).len(),
            advisories,
            "{report}"
        );
    }
}

#[test]
fn cli_names_coverage_owner_uncertainty_in_human_and_json_output() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/coverage-producer-assertion-owner-uncertainty/fixture");
    for (config, expected) in [
        (
            "dispatch",
            vec!["unresolved dispatch", "src/producer.mts:2", "`dispatch`"],
        ),
        (
            "depth",
            vec!["traversal limit maxDepth=1", "last module src/helper.mts"],
        ),
        (
            "nodes",
            vec!["traversal limit maxNodes=1", "last module src/producer.mts"],
        ),
        (
            "mixed",
            vec![
                "unresolved dispatch",
                "`dispatch`",
                "traversal limit maxDepth=1",
            ],
        ),
        (
            "both",
            vec![
                "traversal limit maxDepth=1",
                "traversal limit maxNodes=2",
                "last module src/helper.mts",
            ],
        ),
        (
            "custom",
            vec![
                "Assert the dispatch result",
                "unresolved dispatch",
                "`dispatch`",
            ],
        ),
        (
            "source",
            vec!["source failure", "src/broken.mts", "Unexpected token"],
        ),
    ] {
        for format in ["human", "json"] {
            let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(format!("{config}.yml")))
                .args(["--format", format])
                .output()
                .unwrap();
            assert!(!output.status.success());
            let text = if format == "json" {
                let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                let field = if config == "source" {
                    "incomplete"
                } else {
                    "rules"
                };
                report[field][0]["message"].as_str().unwrap().to_string()
            } else {
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )
            };
            for part in &expected {
                assert!(
                    text.contains(part),
                    "{config}/{format}: missing {part}: {text}"
                );
            }
            assert!(!text.contains("unresolved dispatch, source failure, or traversal limit"));
        }
    }
}

fn check_json(root: &std::path::Path, config: &str) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(format!("{config}.yml")))
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn cli_reports_only_blocking_sites_with_structured_causes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/coverage-producer-assertion-owner-builtins/fixture");
    for (config, callee, expected_causes, more) in [
        ("many", "a", 8, true),
        ("third-party", "Sentry.captureException", 1, false),
        ("callback-unresolved", "items.map", 1, false),
    ] {
        let report = check_json(&root, config);
        let finding = &report["rules"][0];
        let causes = finding["causes"].as_array().unwrap();
        assert_eq!(causes.len(), expected_causes, "{config}: {report}");
        assert_eq!(causes[0]["kind"], "unresolved-dispatch");
        assert_eq!(causes[0]["callee"], callee);
        assert_eq!(causes[0]["file"], "src/producer.mts");
        assert!(causes[0]["line"].as_u64().unwrap() > 0);
        let message = finding["message"].as_str().unwrap();
        assert_eq!(message.contains("+3 more"), more, "{message}");
        assert!(!message.contains("JSON.stringify"), "{message}");
    }
    for config in ["pass", "no-owner"] {
        let report = check_json(&root, config);
        for finding in report["rules"].as_array().unwrap() {
            assert!(finding.get("causes").is_none(), "{report}");
            assert!(!finding["message"]
                .as_str()
                .unwrap()
                .contains("unanalyzable"));
        }
    }
}

#[test]
fn proven_assertion_owner_survives_unresolved_cleanup_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(
        "coverage-producer-assertion-owner-cleanup",
        "fixture",
        "",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    let targets: Vec<_> = findings
        .iter()
        .map(|row| row["target"].as_str().unwrap())
        .collect();
    assert_eq!(
        targets,
        [
            "loggerOnly",
            "mapOnly",
            "flatMapOnly",
            "sentryOnly",
            "cleanupOnly",
            "withoutAssertion",
            "noOwner"
        ],
        "{report}"
    );
    for finding in findings.iter().filter(|row| row["target"] != "noOwner") {
        assert!(
            finding["message"]
                .as_str()
                .unwrap()
                .contains("unanalyzable coverage producer"),
            "{finding}"
        );
        assert!(
            !finding["causes"].as_array().unwrap().is_empty(),
            "{finding}"
        );
    }
    crate::check_json_snapshot::assert_saved_fixture(
        "coverage_producer_assertion_owner_cleanup",
        &[("cleanup", root.clone(), root.join(".no-mistakes.yml"))],
    );
}

#[test]
fn opaque_shutdown_receivers_keep_proven_assertions_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(
        "coverage-producer-assertion-owner-opaque-shutdown",
        "fixture",
        "",
    );
    let report = check_json(&root, ".no-mistakes");
    let findings = report["rules"].as_array().unwrap();
    let targets: Vec<_> = findings
        .iter()
        .map(|finding| finding["target"].as_str().unwrap())
        .collect();
    assert_eq!(
        targets,
        [
            "unownedThen",
            "unownedCatch",
            "unownedFinally",
            "unresolvedThen",
            "unresolvedCatch",
            "unresolvedFinally"
        ],
        "{report}"
    );
    for (target, callee) in [
        ("unresolvedThen", "<unknown>.then"),
        ("unresolvedCatch", "<unknown>.catch"),
        ("unresolvedFinally", "<unknown>.finally"),
    ] {
        let finding = findings
            .iter()
            .find(|finding| finding["target"] == target)
            .unwrap();
        assert_eq!(finding["causes"][0]["callee"], callee, "{finding}");
        assert_eq!(
            finding["causes"][0]["detail"],
            "built-in callback `shutdownStore` is not statically resolved",
            "{finding}"
        );
    }
    let script_report = check_json(&root, "script-roots");
    let script_targets: Vec<_> = script_report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["target"].as_str().unwrap())
        .collect();
    assert_eq!(
        script_targets,
        [
            "runUnresolvedThen",
            "runUnresolvedCatch",
            "runUnresolvedFinally"
        ],
        "{script_report}"
    );
    crate::check_json_snapshot::assert_saved_fixture(
        "coverage_producer_assertion_owner_opaque_shutdown",
        &[
            (
                "proven_direct_owners",
                root.clone(),
                root.join(".no-mistakes.yml"),
            ),
            (
                "caller_only_owners",
                root.clone(),
                root.join("script-roots.yml"),
            ),
        ],
    );
}

#[test]
fn production_scenario_table_reaches_default_member_assertions() {
    let root = crate::check_json_snapshot::rules_path(
        "coverage-producer-assertion-owner-scenario-table",
        "fixture",
        "",
    );
    for config in [
        "direct-default",
        "direct-object",
        "inline-table",
        "global-private",
        ".no-mistakes",
    ] {
        let report = check_json(&root, config);
        assert!(
            report["rules"].as_array().unwrap().is_empty(),
            "{config}: {report}"
        );
        assert!(
            report["advisories"].as_array().unwrap().is_empty(),
            "{config}: {report}"
        );
    }
}

#[test]
fn scenario_table_guards_and_policy_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(
        "coverage-producer-assertion-owner-scenario-table",
        "fixture",
        "",
    );
    let report = check_json(&root, "guards");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    let targets: Vec<_> = report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["target"].as_str().unwrap())
        .collect();
    assert_eq!(
        targets,
        [
            "dynamic",
            "mutated",
            "escaped",
            "empty",
            "uninvoked",
            "firstDestructured",
            "earlyBreak",
            "earlyReturn",
            "filtered",
            "shadowed",
            "overwrittenRow",
            "aliasedTable",
            "malformedRow",
            "globalMutated",
            "globalEscaped",
            "captured",
            "earlierEscapeLoop",
            "earlierMutationLoop"
        ],
        "{report}"
    );
    let cases: Vec<_> = [
        ".no-mistakes",
        "direct-default",
        "direct-object",
        "inline-table",
        "global-private",
        "no-owner",
        "guards",
    ]
    .into_iter()
    .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))))
    .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "coverage_producer_assertion_owner_scenario_table",
        &cases,
    );
}
