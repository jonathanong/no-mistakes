use std::path::PathBuf;
use std::process::Command;

#[test]
fn effects_cli_preserves_exported_receiver_live_and_copied_acquisitions() {
    for (fixture, entry, expected) in [
        ("exact-fixture", "entry", Vec::<&str>::new()),
        (
            "fixture",
            "entry",
            vec!["other.get", "shared.get", "shared.set"],
        ),
        (
            "fixture",
            "barrel-entry",
            vec!["other.get", "shared.get", "shared.set"],
        ),
    ] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/value-flow-exported-value-receiver")
            .join(fixture);
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args([
                "effects",
                "receiver",
                "--entry",
                &format!("src/{entry}.mts"),
                "--root",
            ])
            .arg(&root)
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let mut callees: Vec<_> = report["callSites"]
            .as_array()
            .unwrap()
            .iter()
            .map(|site| site["callee"].as_str().unwrap())
            .collect();
        callees.sort_unstable();
        assert_eq!(callees, expected, "{fixture}/{entry}: {report:#?}");
    }
}
