use serde_json::Value;

#[test]
fn nested_require_reports_the_inner_load_only() {
    let root = crate::check_json_snapshot::rules_path(
        "test-no-live-repository-scan",
        "nested-requires",
        "",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    let advisories = report["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 1, "{report}");
    assert_eq!(advisories[0]["file"], "tests/nested.test.ts");
    assert_eq!(advisories[0]["line"], 4);
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_nested_requires",
        &[("nested", root.clone(), root.join(".no-mistakes.yml"))],
    );
}

#[test]
fn remaining_enumeration_shapes_do_not_silently_pass() {
    let root =
        crate::check_json_snapshot::rules_path("test-no-live-repository-scan", "fixture", "");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let source = std::fs::read_to_string(root.join("tests/remaining-shapes.test.ts")).unwrap();
    let line = |marker: &str| {
        source
            .lines()
            .position(|line| line.ends_with(&format!("// {marker}")))
            .unwrap()
            + 1
    };
    let has = |field: &str, marker: &str| {
        report[field].as_array().unwrap().iter().any(|finding| {
            finding["file"] == "tests/remaining-shapes.test.ts" && finding["line"] == line(marker)
        })
    };
    for marker in [
        "shared-temp",
        "shared-child",
        "escaped-fixture",
        "meta-live",
        "meta-eager",
        "extra-live",
        "extra-named",
    ] {
        assert!(has("rules", marker), "missing {marker}: {report}");
    }
    for marker in ["meta-unresolved", "meta-array-unresolved"] {
        assert!(has("advisories", marker), "missing {marker}: {report}");
    }
    for marker in [
        "nested-fixture",
        "meta-fixture",
        "extra-fixture",
        "local-glob",
    ] {
        assert!(
            !has("rules", marker) && !has("advisories", marker),
            "{marker}: {report}"
        );
    }
}
