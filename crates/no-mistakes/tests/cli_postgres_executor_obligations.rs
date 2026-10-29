use serde_json::Value;

#[test]
fn configured_executor_runtime_references_reach_every_unanalyzable_policy() {
    let root = crate::check_json_snapshot::rules_path(
        "postgres-no-test-state-mutation",
        "detached-obligations",
        "",
    );
    for effort in ["low", "max"] {
        for policy in ["advise", "report", "ignore"] {
            let case = if policy == "advise" {
                effort.to_string()
            } else {
                format!("{effort}-{policy}")
            };
            for jobs in ["1", "2"] {
                let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                    .args(["check", "--root"])
                    .arg(&root)
                    .arg("--config")
                    .arg(root.join(format!("{case}.yml")))
                    .args(["--jobs", jobs, "--format", "json"])
                    .output()
                    .unwrap();
                let report: Value = serde_json::from_slice(&output.stdout).unwrap();
                assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
                let rows = |field: &str, file: &str, line: u32| {
                    report[field]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|row| row["file"] == file && row["line"] == line)
                };
                for (file, lines) in [
                    (
                        "tests/references.test.ts",
                        vec![3, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 19, 20, 21, 22, 24],
                    ),
                    ("tests/names.test.ts", vec![3, 5, 6, 7, 9, 10]),
                    ("tests/wrapped-reference.test.ts", vec![2]),
                    ("tests/forward-export.test.ts", vec![2]),
                    ("tests/parameter-alias.test.ts", vec![2, 3, 6, 7]),
                ] {
                    for line in lines {
                        assert_eq!(
                            rows("advisories", file, line),
                            policy == "advise",
                            "{case} {file}:{line}: {report}"
                        );
                        assert_eq!(
                            rows("rules", file, line),
                            policy == "report",
                            "{case} {file}:{line}: {report}"
                        );
                    }
                }
                for line in [3, 4, 5, 6] {
                    assert!(
                        rows("rules", "tests/known.test.ts", line),
                        "{case}: {report}"
                    );
                }
                for line in [2, 4] {
                    let uncertain = effort == "low";
                    assert_eq!(
                        rows("rules", "tests/direct-wrappers.test.ts", line),
                        !uncertain || policy == "report",
                        "{case}: {report}"
                    );
                    assert_eq!(
                        rows("advisories", "tests/direct-wrappers.test.ts", line),
                        uncertain && policy == "advise",
                        "{case}: {report}"
                    );
                }
                for field in ["rules", "advisories"] {
                    assert!(
                        !report[field]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|row| row["rule"] == "postgres-no-test-state-mutation"
                                && [
                                    "tests/controls.test.ts",
                                    "tests/type-uses.test.ts",
                                    "tests/suppressed.test.ts",
                                    "tests/file-suppressed.test.ts"
                                ]
                                .iter()
                                .any(|file| row["file"] == *file)),
                        "{case}: {report}"
                    );
                }
            }
        }
    }
    let cases = [
        "low",
        "max",
        "low-report",
        "max-report",
        "low-ignore",
        "max-ignore",
    ];
    let cases = cases
        .iter()
        .map(|case| (*case, root.clone(), root.join(format!("{case}.yml"))))
        .collect::<Vec<_>>();
    crate::check_json_snapshot::assert_saved_fixture(
        "postgres_executor_reference_obligations",
        &cases,
    );
}
