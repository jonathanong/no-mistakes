use std::path::PathBuf;
use std::process::Command;

#[test]
fn storybook_effort_max_reports_at_least_as_much_as_low() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/require-storybook-stories/fixture/effort");
    for (config, findings, advisories) in [
        (".no-mistakes.max.yml", 3, 0),
        (".no-mistakes.low.yml", 3, 0),
        (".no-mistakes.low-advise.yml", 1, 2),
        (".no-mistakes.low-ignore.yml", 1, 0),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(config))
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{config}: {report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{config}: {report}"
        );
        assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    }
}
