use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_targets_vitest_uses_static_object_project_labels() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-projects-target");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "test",
            "targets",
            "vitest",
            "tests/unit.test.ts",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let targets = value["tests"][0]["targets"].as_array().unwrap();
    let projects = targets
        .iter()
        .filter_map(|target| target["project"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(projects, ["inline-object", "nested-object"]);
}
