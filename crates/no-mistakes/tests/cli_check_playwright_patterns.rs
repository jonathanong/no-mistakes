use std::path::PathBuf;
use std::process::Command;

#[test]
fn playwright_matchers_keep_json_reports_and_unrelated_findings() {
    for (name, unknown) in [
        ("regexp-match", None),
        ("mixed-regexp-match", None),
        ("string-match", None),
        ("unknown-match", Some("testMatch")),
        ("unsupported-regexp-match", Some("testMatch")),
        ("unknown-ignore", Some("testIgnore")),
    ] {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/playwright-configs")
            .join(name)
            .join("fixture");
        let fixture = crate::saved_fixture::materialize_path(&source);
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["check", "--format", "json", "--root"])
            .arg(fixture.path())
            .output()
            .unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                panic!(
                    "{name}: {error}: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
            });
        // This definite finding proves a failed matcher never aborts other rules.
        assert_eq!(output.status.code(), Some(1), "{name}: {report}");
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{name}: {report}");
        assert_eq!(findings[0]["rule"], "file-extension-policy");
        assert_eq!(findings[0]["file"], "forbidden.txt");
        assert_eq!(report["advisories"], serde_json::json!([]));
        let warnings = report["warnings"].as_array().unwrap();
        if let Some(property) = unknown {
            assert!(!warnings.is_empty(), "{name}: {report}");
            assert!(
                warnings.iter().any(|warning| {
                    let warning = warning.as_str().unwrap();
                    warning.contains("playwright.config.ts:5") && warning.contains(property)
                }),
                "{name}: {report}"
            );
        } else {
            assert!(warnings.is_empty(), "{name}: {report}");
        }
    }
}

#[test]
fn standalone_playwright_coverage_exposes_matcher_uncertainty() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/playwright-configs/unknown-match/fixture");
    let fixture = crate::saved_fixture::materialize_path(&source);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["playwright", "--root"])
        .arg(fixture.path())
        .args(["check", "--json"])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
    assert_eq!(output.status.code(), Some(1), "{report}");
    assert!(
        report["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| { warning.as_str().unwrap().contains("playwright.config.ts:5") }),
        "{report}"
    );
}

#[test]
fn runner_only_playwright_policies_keep_scoped_warnings_and_other_rules() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/playwright-configs/unknown-match/fixture");
    let fixture = crate::saved_fixture::materialize_path(&source);
    for (config, uncertain) in [
        ("integration-only.yml", true),
        ("known-integration.yml", false),
    ] {
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
        assert_eq!(report["rules"].as_array().unwrap().len(), 1, "{report}");
        assert_eq!(report["rules"][0]["rule"], "file-extension-policy");
        assert_eq!(
            report["warnings"].as_array().unwrap().is_empty(),
            !uncertain,
            "{config}: {report}"
        );
        if uncertain {
            assert!(
                report["warnings"]
                    .to_string()
                    .contains("playwright.config.ts:5"),
                "{report}"
            );
        }
    }
}
