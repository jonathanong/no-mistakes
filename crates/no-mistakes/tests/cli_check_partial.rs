use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check")
        .join(name)
        .canonicalize()
        .unwrap()
}

fn check(name: &str, config: &str, jobs: &str) -> (i32, serde_json::Value) {
    let root = fixture(name);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--jobs", jobs, "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .args(["--format", "json", "--timeout", "0"])
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
    (output.status.code().unwrap(), report)
}

#[test]
fn failed_source_retains_independent_findings_and_reports_consumer() {
    for jobs in ["1", "2"] {
        let (exit, report) = check("partial-source", ".no-mistakes.yml", jobs);
        assert_ne!(exit, 0);
        let findings = report["rules"].as_array().unwrap();
        assert!(findings
            .iter()
            .any(|finding| finding["rule"] == "no-empty-or-comments-only-files"));
        assert!(findings
            .iter()
            .any(|finding| finding["rule"] == "nextjs-no-api-routes"));
        let failures = report["incomplete"].as_array().unwrap();
        assert!(
            failures.iter().any(|failure| {
                failure["consumer"] == "forbidden-calls"
                    && failure.to_string().contains("template.js")
            }),
            "{report}"
        );
    }
}

#[test]
fn unsupported_config_retains_filesystem_findings_and_names_property() {
    for jobs in ["1", "2"] {
        let (exit, report) = check("partial-playwright", ".no-mistakes.yml", jobs);
        assert_ne!(exit, 0);
        assert!(report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["rule"] == "no-empty-or-comments-only-files" }));
        assert!(
            report["incomplete"]
                .as_array()
                .unwrap()
                .iter()
                .any(|failure| {
                    failure["consumer"] == "playwright-coverage"
                        && failure.to_string().contains("playwright.config.ts")
                        && failure.to_string().contains("testDir")
                }),
            "{report}"
        );
    }
}

#[test]
fn unsupported_vitest_include_names_its_config_property_and_line() {
    for jobs in ["1", "2"] {
        let (exit, report) = check("partial-vitest", ".no-mistakes.yml", jobs);
        assert_ne!(exit, 0);
        assert!(report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["rule"] == "no-empty-or-comments-only-files" }));
        let failures = report["incomplete"].as_array().unwrap();
        for consumer in [
            "vitest-project-mapping",
            "vitest-ci-path-coverage",
            "test-no-unmocked-dynamic-imports",
        ] {
            let failure = failures
                .iter()
                .find(|failure| failure["consumer"] == consumer)
                .unwrap_or_else(|| panic!("{report}"));
            assert!(failure["file"]
                .as_str()
                .unwrap()
                .ends_with("vitest.config.ts"));
            assert_eq!(failure["property"], "test.include");
            assert!(failure["message"]
                .as_str()
                .unwrap()
                .contains("vitest.config.ts:1: test.include"));
        }
    }
}

#[test]
fn failures_outside_consumer_scope_do_not_mark_successful_checks_incomplete() {
    for name in ["partial-source", "partial-playwright"] {
        let (_, report) = check(name, "valid.yml", "1");
        assert!(report.get("incomplete").is_none(), "{report}");
    }
}

#[test]
fn disabled_parse_failure_does_not_discard_independent_unique_export_application() {
    let (exit, report) = check("partial-unique", ".no-mistakes.yml", "2");
    assert_ne!(exit, 0);
    assert!(
        report["incomplete"]
            .as_array()
            .unwrap()
            .iter()
            .any(|failure| {
                failure["consumer"] == "unique-exports"
                    && failure["file"]
                        .as_str()
                        .is_some_and(|file| file.ends_with("bad/broken.ts"))
            }),
        "{report}"
    );
    assert!(
        report["codebase"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["exportName"] == "duplicate" }),
        "{report}"
    );
}

#[test]
fn failed_sql_helper_retains_independent_findings() {
    for jobs in ["1", "2"] {
        let (exit, report) = check("sql-helper-prerequisite", ".no-mistakes.yml", jobs);
        assert_ne!(exit, 0);
        assert!(
            report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .any(|finding| { finding["rule"] == "no-empty-or-comments-only-files" }),
            "{report}"
        );
        assert!(
            report["incomplete"]
                .as_array()
                .unwrap()
                .iter()
                .any(|failure| {
                    failure["consumer"] == "postgres-require-query-annotation"
                        && failure["file"]
                            .as_str()
                            .is_some_and(|file| file.ends_with("helper.ts"))
                }),
            "{report}"
        );
    }
}

#[test]
fn check_json_snapshot_partial_prerequisites() {
    let mut cases = [
        "partial-source",
        "partial-playwright",
        "partial-vitest",
        "partial-unique",
        "coverage-owner-partial",
        "sql-helper-prerequisite",
    ]
    .map(|name| {
        let root = fixture(name);
        (name, root.clone(), root.join(".no-mistakes.yml"))
    })
    .to_vec();
    let root = fixture("../playwright/mixed-selection-prerequisites");
    cases.push((
        "mixed-playwright-selection",
        root.clone(),
        root.join(".no-mistakes.yml"),
    ));
    let root = fixture("storybook-partial");
    for (label, config) in [
        ("storybook-source-allowances", "allowances.yml"),
        ("storybook-catalog-allowances", "catalog-allowances.yml"),
    ] {
        cases.push((label, root.clone(), root.join(config)));
    }
    crate::check_json_snapshot::assert_saved_fixture("partial_prerequisites", &cases);
}
