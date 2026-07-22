#[allow(dead_code)]
#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

#[test]
fn resource_change_plan_json_keeps_resource_call_site_provenance() {
    // Use a copied fixture root: repository inventories intentionally omit the
    // worktree's untracked fixture additions while they are under development.
    let fixture = gitignore_fixture::materialize_saved("../test-plan/resource-impact");
    let root = fixture.path();
    let output = Command::new(bin())
        .args(["test", "plan", "--root"])
        .arg(root)
        .args(["--changed-file", "resources/page.txt", "--json"])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let selected = plan["selected_tests"]
        .as_array()
        .expect("plan JSON must include selected tests");
    assert_eq!(selected.len(), 1, "{plan:#?}");
    assert_eq!(selected[0]["test_file"], "impact-consumer.test.ts");

    let reason = &selected[0]["reasons"][0];
    assert_eq!(reason["via"], serde_json::json!(["resource", "dependency"]));
    assert_eq!(reason["via_details"][0]["type"], "resource");
    assert_eq!(
        reason["via_details"][0]["consumer_file"],
        "impact-consumer.ts"
    );
    assert_eq!(
        reason["via_details"][0]["call_sites"],
        serde_json::json!([{"call_kind": "read-file-sync", "line": 3}])
    );
}
