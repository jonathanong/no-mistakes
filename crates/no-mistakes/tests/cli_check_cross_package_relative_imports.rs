use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "no-cross-package-relative-imports";

#[test]
fn cross_package_relative_imports_match_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    let reports = assert_saved_fixture_report(
        "cross_package_relative_imports",
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
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect();
    locations.sort_unstable();
    assert_eq!(
        locations,
        [
            ("packages/app/src/crossings.mts", 1),
            ("packages/app/src/crossings.mts", 2),
            ("packages/app/src/crossings.mts", 3),
            ("packages/app/src/crossings.mts", 4),
            ("packages/app/src/crossings.mts", 5),
            ("packages/app/src/crossings.mts", 9),
            ("packages/app/src/legacy.cjs", 1),
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
    assert!(failures
        .iter()
        .any(|row| row["consumer"] == RULE && row["file"] == "packages/app/src/gaps/broken.mts"));
    let findings = incomplete["report"]["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["file"], "packages/app/src/gaps/healthy.mts");
    assert_eq!(findings[0]["line"], 1);
}

#[test]
fn cross_package_relative_imports_reports_opaque_vitest_helpers() {
    let root = rules_path(RULE, "fixture", "");
    let cases = ["default", "report", "ignore"].map(|policy| {
        (
            policy,
            root.clone(),
            root.join(format!("helper-vitest-{policy}.yml")),
        )
    });
    let reports = assert_saved_fixture_report(
        "cross_package_relative_imports_vitest_helpers",
        &cases,
        &[],
        &[],
    );

    let advised = &reports["default"];
    assert_eq!(advised["exitCode"], 0);
    let findings = advised["report"]["advisories"].as_array().unwrap();
    let calls: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            assert!(finding.get("target").is_none());
            (
                finding["line"].as_u64().unwrap(),
                finding["import"].as_str().unwrap(),
            )
        })
        .collect();
    let crossing = "../../shared/src/status.mts";
    assert_eq!(
        calls,
        [
            (5, crossing),
            (6, crossing),
            (8, crossing),
            (9, "<computed>")
        ]
    );

    let reported = &reports["report"];
    assert_eq!(reported["exitCode"], 1);
    assert_eq!(reported["report"]["rules"], advised["report"]["advisories"]);
    assert_eq!(reported["report"]["advisories"], serde_json::json!([]));

    let ignored = &reports["ignore"];
    assert_eq!(ignored["exitCode"], 0);
    assert_eq!(ignored["report"]["rules"], serde_json::json!([]));
    assert_eq!(ignored["report"]["advisories"], serde_json::json!([]));
}
