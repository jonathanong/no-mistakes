use std::path::PathBuf;
use std::process::Command;

#[test]
fn nextjs_delegated_low_cli_keeps_missing_destinations_and_policy_channels() {
    for name in ["low-reexport", "low-path-alias"] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/nextjs-redirect-destinations/fixture")
            .join(name);
        for policy in ["report", "advise", "ignore"] {
            let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(root.join(format!(".no-mistakes-low-{policy}.yml")))
                .args(["--format", "json"])
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(1),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(
                report["rules"].as_array().unwrap().len(),
                if policy == "report" { 4 } else { 2 },
                "{report}"
            );
            assert_eq!(
                report["advisories"].as_array().unwrap().len(),
                if policy == "advise" { 2 } else { 0 },
                "{report}"
            );
            assert!(report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row["file"] == "next-shared.ts"));
        }
    }
}
