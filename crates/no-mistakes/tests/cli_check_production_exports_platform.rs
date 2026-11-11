use serde_json::Value;
use std::process::Command;

fn report(config: &str) -> Value {
    let root = crate::check_json_snapshot::rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "platform-consumed",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .args(["--config"])
        .arg(root.join(config))
        .args(["--format", "json"])
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn platform_consumed_exports_are_exact_and_keep_sibling_uncertainty() {
    let baseline = report(".no-mistakes.yml");
    assert_eq!(baseline, report("empty.yml"));
    let enabled = report("enabled.yml");
    let findings = enabled["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{enabled}");
    assert_eq!(findings[0]["target"], "stillTestOnly");
    assert_eq!(findings[1]["file"], "src/other.mts");
    assert_eq!(findings[1]["target"], "Worker");
    assert_eq!(baseline["rules"].as_array().unwrap().len(), 4);
    let advisories = enabled["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 1, "{enabled}");
    assert_eq!(advisories[0]["file"], "tests/helper.mts");
    assert!(!baseline["advisories"].as_array().unwrap().is_empty());
}

#[test]
fn platform_consumed_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "platform-consumed",
    );
    let cases: Vec<_> = [
        ".no-mistakes.yml",
        "enabled.yml",
        "empty.yml",
        "off.yml",
        "invalid-file.yml",
        "invalid-names.yml",
        "invalid-name.yml",
        "invalid-parent.yml",
        "invalid-absolute.yml",
        "invalid-field.yml",
    ]
    .into_iter()
    .map(|name| (name, root.clone(), root.join(name)))
    .collect();
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_platform_consumed",
        &cases,
    );
}

#[test]
fn external_star_exports_check_json_snapshot() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/external-star-exports/fixture");
    let controls = root.join("../../external-star-exports-controls/fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&controls)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let targets: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["target"].as_str().unwrap())
        .collect();
    assert_eq!(targets, ["testOnly", "Queue"], "{report}");
    assert_external_star_snapshot(&root, &controls);
}

fn assert_external_star_snapshot(root: &std::path::Path, controls: &std::path::Path) {
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_external_star",
        &[
            ("enabled", root.to_path_buf(), root.join(".no-mistakes.yml")),
            ("off", root.to_path_buf(), root.join("off.yml")),
            (
                "uncertain_controls",
                controls.to_path_buf(),
                controls.join(".no-mistakes.yml"),
            ),
        ],
    );
}

#[test]
fn external_star_exports_check_json_snapshot_without_git() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/external-star-exports/fixture");
    let controls = crate::saved_fixture::materialize_path(
        &root.join("../../external-star-exports-controls/fixture"),
    );
    assert!(!controls.path().join(".git").exists());
    assert!(controls.path().join("local-owned.mts").is_file());
    assert!(controls
        .path()
        .join("node_modules/local-owned/package.json")
        .is_file());
    assert!(controls
        .path()
        .join("node_modules/local-owned/entry.js")
        .is_file());
    assert_external_star_snapshot(&root, controls.path());
}

#[test]
fn platform_consumed_reexports_check_json_snapshot() {
    let root = crate::check_json_snapshot::rules_path(
        "production-exports-only-used-by-tests",
        "fixture",
        "platform-reexports",
    );
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_platform_reexports",
        &[("reexports", root.clone(), root.join(".no-mistakes.yml"))],
    );
}
