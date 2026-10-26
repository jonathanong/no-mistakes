use std::{path::PathBuf, process::Command};

#[test]
fn unavailable_version_pin_source_policy_routes_cli_channels() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/version-pin-consistency/fixture/unavailable-source");
    for prefix in ["", "missing-", "mixed-"] {
        for (policy, uncertain, advisories) in
            [("report", 1, 0), ("advise", 0, 1), ("ignore", 0, 0)]
        {
            let ordinary = usize::from(prefix == "mixed-");
            let out = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(format!("{prefix}{policy}.yml")))
                .args(["--format", "json"])
                .output()
                .unwrap();
            assert_eq!(
                out.status.code(),
                Some(i32::from(uncertain + ordinary > 0)),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(
                value["rules"].as_array().unwrap().len(),
                uncertain + ordinary,
                "{value}"
            );
            assert_eq!(
                value["advisories"].as_array().unwrap().len(),
                advisories,
                "{value}"
            );
            assert_eq!(value["warnings"], serde_json::json!([]));
        }
    }
}
