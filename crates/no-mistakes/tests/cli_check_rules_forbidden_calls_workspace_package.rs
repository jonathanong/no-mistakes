#[path = "common/git_tracked_rule.rs"]
mod git_tracked_rule;

use serde_json::Value;
use std::process::Command;

#[test]
fn banned_functions_called_through_workspace_package_names_are_reported() {
    let root = git_tracked_rule::materialize_rule("forbidden-calls", "workspace-package");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(root.path())
        .arg("--config")
        .arg(root.path().join(".no-mistakes.yml"))
        .output()
        .unwrap();
    let body = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    let findings = report["rules"].as_array().map(Vec::as_slice).unwrap_or(&[]);

    assert_eq!(output.status.code(), Some(1), "{body}");
    // `danger` comes from the package root and `subDanger` from its `exports`
    // subpath; the allowed `safe` call between them is not reported.
    let reported = findings
        .iter()
        .map(|finding| {
            (
                finding["file"].as_str().unwrap_or_default(),
                finding["line"].as_u64().unwrap_or_default(),
                finding["import"].as_str().unwrap_or_default(),
                finding["target"].as_str().unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        reported,
        vec![
            (
                "src/entry.ts",
                6,
                "danger",
                "repository function `packages/lib/index.ts#danger`"
            ),
            (
                "src/entry.ts",
                7,
                "subDanger",
                "repository function `packages/lib/src/sub.ts#subDanger`"
            ),
        ],
        "{body}"
    );
}
