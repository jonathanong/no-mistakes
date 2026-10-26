use std::path::PathBuf;
use std::process::Command;

#[test]
fn playwright_runner_only_unknown_matchers_keep_scoped_warnings() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/playwright-configs/runner-only-unknown-match/fixture");
    for (config, unknown, timer_count) in [
        (".no-mistakes.yml", true, 0),
        ("all.no-mistakes.yml", true, 1),
        ("sibling.no-mistakes.yml", false, 1),
        ("explicit.no-mistakes.yml", false, 1),
    ] {
        let fixture = crate::saved_fixture::materialize_path(&source);
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(fixture.path())
            .arg("--config")
            .arg(fixture.path().join(config))
            .output()
            .unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                panic!(
                    "{config}: {error}: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
            });
        assert_eq!(output.status.code(), Some(1), "{config}: {report}");
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(
            findings
                .iter()
                .filter(|finding| finding["rule"] == "file-extension-policy")
                .count(),
            1,
            "{config}: {report}"
        );
        assert_eq!(
            findings
                .iter()
                .filter(|finding| finding["rule"] == "forbidden-calls")
                .count(),
            timer_count,
            "{config}: {report}"
        );
        let warnings = report["warnings"].as_array().unwrap();
        assert_eq!(!warnings.is_empty(), unknown, "{config}: {report}");
        if unknown {
            assert!(
                warnings.iter().any(|warning| warning
                    .as_str()
                    .unwrap()
                    .contains("playwright.config.ts:5")
                    && warning.as_str().unwrap().contains("testMatch")),
                "{config}: {report}"
            );
        }
    }
}
