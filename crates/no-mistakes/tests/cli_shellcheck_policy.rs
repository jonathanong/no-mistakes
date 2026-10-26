use std::path::PathBuf;
use std::process::Command;

fn check(config: &str) -> (i32, serde_json::Value) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/shellcheck-runner/fixture/unavailable");
    // Only the child loses PATH: other tests can still run Git and ShellCheck.
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .args(["--format", "json"])
        .env("PATH", "")
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {:?}", output));
    (output.status.code().unwrap(), report)
}

#[test]
fn shellcheck_missing_executable_routes_selected_scripts_through_policy_at_max() {
    for (config, exit, failures, advisories) in [
        ("report.yml", 1, 2, 0),
        ("advise.yml", 0, 0, 2),
        ("ignore.yml", 0, 0, 0),
        ("no-selected.yml", 0, 0, 0),
        ("rule-ignore.yml", 0, 0, 0),
        ("scoped.yml", 1, 1, 0),
        ("custom.yml", 0, 0, 2),
    ] {
        let (status, report) = check(config);
        assert_eq!(status, exit, "{config}: {report}");
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            failures,
            "{config}: {report}"
        );
        assert_eq!(
            super::policy_advisories::without_hygiene(&report).len(),
            advisories,
            "{config}: {report}"
        );
        assert_eq!(report["warnings"], serde_json::json!([]));
        for finding in report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .chain(super::policy_advisories::without_hygiene(&report))
        {
            assert_eq!(finding["target"], "unanalyzable");
            assert_eq!(finding["line"], 1);
            assert!(matches!(
                finding["file"].as_str().unwrap(),
                "scripts/check.sh" | "scripts/next-disabled.sh"
            ));
            if config == "custom.yml" {
                assert_eq!(
                    finding["message"],
                    "Install the project ShellCheck dependency."
                );
            }
        }
    }
}
