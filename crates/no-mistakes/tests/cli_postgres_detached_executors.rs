use serde_json::Value;

fn root() -> std::path::PathBuf {
    crate::check_json_snapshot::rules_path(
        "postgres-no-test-state-mutation",
        "detached-executors",
        "",
    )
}

#[test]
fn detached_executor_calls_are_not_silent() {
    let root = root();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("max.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    for line in [4, 7, 10, 12, 14, 17, 19] {
        assert!(
            findings
                .iter()
                .any(|finding| finding["file"] == "tests/exact.test.ts" && finding["line"] == line),
            "detached executor at line {line} was lost: {report}"
        );
    }
    assert!(
        findings
            .iter()
            .any(|finding| finding["file"] == "tests/after-copy.test.ts" && finding["line"] == 7),
        "MAX must preserve the original copy's event through aliases: {report}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding["file"] == "tests/local-bodies.test.ts" && finding["line"] == 5),
        "{report}"
    );
    assert!(
        report["advisories"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| {
                finding["file"] == "tests/local-bodies.test.ts" && finding["line"] == 11
            }),
        "{report}"
    );
    assert!(
        !findings.iter().any(|finding| {
            finding["file"] == "tests/local-bodies.test.ts" && finding["line"] == 11
        }),
        "{report}"
    );
    for (file, line) in [
        ("mixed-roles.test.ts", 5),
        ("mixed-roles.test.ts", 6),
        ("mixed-roles.test.ts", 9),
        ("mixed-roles.test.ts", 10),
        ("assigned.test.ts", 4),
        ("assigned.test.ts", 7),
        ("assigned.test.ts", 10),
        ("assigned.test.ts", 14),
        ("assigned.test.ts", 17),
        ("assigned.test.ts", 20),
        ("assigned.test.ts", 22),
        ("uncertain.test.ts", 7),
        ("wrong-this.test.ts", 3),
        ("prebound.test.ts", 5),
        ("mutable.test.ts", 6),
        ("escaped.test.ts", 5),
        ("receiver-escaped.test.ts", 6),
        ("receiver-history.test.ts", 8),
        ("receiver-origins.test.ts", 8),
        ("receiver-origins.test.ts", 10),
    ] {
        let path = format!("tests/{file}");
        assert!(
            report["advisories"]
                .as_array()
                .unwrap()
                .iter()
                .any(|finding| finding["file"] == path && finding["line"] == line),
            "uncertain detached executor disappeared at {path}:{line}: {report}"
        );
        assert!(
            !findings.iter().any(|finding| finding["file"] == path),
            "uncertain binding must not certify caller SQL: {report}"
        );
    }
    assert!(
        !findings
            .iter()
            .chain(report["advisories"].as_array().unwrap())
            .any(
                |finding| ["tests/controls.test.ts", "tests/callable-controls.test.ts"]
                    .iter()
                    .any(|file| finding["file"] == *file)
            ),
        "{report}"
    );
    crate::check_json_snapshot::assert_saved_fixture(
        "postgres_detached_query_executors",
        &[
            ("max", root.clone(), root.join("max.yml")),
            ("low", root.clone(), root.join("low.yml")),
            ("report", root.clone(), root.join("report.yml")),
            ("ignore", root.clone(), root.join("ignore.yml")),
        ],
    );
}

fn low_report(jobs: &str) -> Value {
    let root = root();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join("low.yml"))
        .args(["--jobs", jobs, "--format", "json"])
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

fn has_site(report: &Value, field: &str, file: &str, line: u32) -> bool {
    report[field]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["file"] == file && row["line"] == line)
}

#[test]
fn low_detached_executors_preserve_scope_uncertainty_and_suppression() {
    for jobs in ["1", "2"] {
        let report = low_report(jobs);
        assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
        for (file, lines) in [
            ("tests/exact.test.ts", vec![4, 7, 10, 12, 14, 17, 19]),
            ("tests/shape-forms.test.ts", vec![3, 5, 8, 10]),
            ("tests/scopes.test.ts", vec![3, 8]),
            ("tests/local-bodies.test.ts", vec![5]),
            ("tests/suppressed.test.ts", vec![6]),
        ] {
            for line in lines {
                assert!(
                    has_site(&report, "rules", file, line),
                    "{file}:{line}: {report}"
                );
            }
        }
        for (file, line) in [
            ("tests/wrong-this.test.ts", 3),
            ("tests/prebound.test.ts", 5),
            ("tests/after-copy.test.ts", 7),
            ("tests/receiver-escaped-after.test.ts", 6),
            ("tests/local-bodies.test.ts", 11),
            ("tests/assigned.test.ts", 4),
            ("tests/assigned.test.ts", 7),
            ("tests/assigned.test.ts", 10),
            ("tests/assigned.test.ts", 14),
            ("tests/assigned.test.ts", 17),
            ("tests/assigned.test.ts", 20),
            ("tests/assigned.test.ts", 22),
            ("tests/mixed-roles.test.ts", 5),
            ("tests/mixed-roles.test.ts", 6),
            ("tests/mixed-roles.test.ts", 9),
            ("tests/mixed-roles.test.ts", 10),
        ] {
            assert!(
                has_site(&report, "advisories", file, line),
                "{file}:{line}: {report}"
            );
            assert!(
                !has_site(&report, "rules", file, line),
                "{file}:{line}: {report}"
            );
        }
        for field in ["rules", "advisories"] {
            assert!(
                !report[field].as_array().unwrap().iter().any(|row| {
                    [
                        "tests/format-bind.test.ts",
                        "tests/controls.test.ts",
                        "tests/file-suppressed.test.ts",
                    ]
                    .iter()
                    .any(|file| row["file"] == *file)
                        || (row["file"] == "tests/suppressed.test.ts"
                            && [4, 5].iter().any(|line| row["line"] == *line))
                }),
                "{report}"
            );
        }
    }
}
