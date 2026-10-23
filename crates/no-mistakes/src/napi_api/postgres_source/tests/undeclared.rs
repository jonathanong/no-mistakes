use super::*;

#[test]
fn diagnostics_are_additive_under_both_extraction_modes() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/embedded/undeclared-executor-types.ts"
    ));
    for resolve in [false, true] {
        let mut request = serde_json::json!({
            "sources": [{"fileName": "query.ts", "source": source}],
            "importSpecifier": "@example/db", "executorNames": ["query", "execute"],
            "executorTypeNames": ["Executor"], "resolveImports": resolve,
        });
        let legacy: serde_json::Value = serde_json::from_str(
            &extract_executed_postgres_sql_json_impl(request.clone()).unwrap(),
        )
        .unwrap();
        request["includeDiagnostics"] = false.into();
        let explicit: serde_json::Value = serde_json::from_str(
            &extract_executed_postgres_sql_json_impl(request.clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(legacy, explicit);
        request["includeDiagnostics"] = true.into();
        let report: serde_json::Value =
            serde_json::from_str(&extract_executed_postgres_sql_json_impl(request).unwrap())
                .unwrap();
        assert_eq!(report["statements"], legacy);
        assert_eq!(legacy.as_array().unwrap().len(), 1);
        assert_eq!(
            report["undeclaredExecutorTypes"],
            serde_json::json!([
                {"type":"PoolClient", "module":"@example/db", "count":6},
                {"type":"PoolClient", "module":"@example/db/client", "count":2}
            ])
        );
        assert_eq!(
            report["unresolvedExecutorReceivers"],
            serde_json::json!([
                {"type":"Executor", "module":"@example/db", "count":1}
            ])
        );
    }
}

#[test]
fn rust_report_preserves_legacy_statements_and_import_selection() {
    use crate::codebase::postgres::*;
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/executed-source/reads.ts"
    ));
    let sources = [ExecutedPostgresSource {
        file_name: "reads.ts".into(),
        source: source.into(),
    }];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    for imports in [None, Some([].as_slice())] {
        let report = extract_executed_postgres_sql_report(&sources, &options, imports).unwrap();
        let legacy = if let Some(imports) = imports {
            extract_executed_postgres_sql_with_imports(&sources, &options, imports).unwrap()
        } else {
            extract_executed_postgres_sql(&sources, &options).unwrap()
        };
        assert_eq!(report.statements, legacy);
        assert!(report.undeclared_executor_types.is_empty());
        assert!(report.unresolved_executor_receivers.is_empty());
    }
}

#[test]
fn undeclared_executor_policies_and_suppressions_reach_public_check_reports() {
    use crate::check_runner::{run_all_with_options, AnalysisOptions};
    use crate::config::v2::Effort;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/undeclared-executors",
    );
    for effort in [Effort::Max, Effort::Low] {
        for (policy, failures, advisories) in [
            ("report", 1, 0),
            ("advise", 0, 1),
            ("ignore", 0, 0),
            ("declared", 0, 0),
        ] {
            let report = run_all_with_options(
                root.clone(),
                Some(format!("{policy}.yml").into()),
                None,
                false,
                false,
                AnalysisOptions {
                    effort: Some(effort),
                    include_sql_verdicts: false,
                },
            )
            .unwrap();
            assert_eq!(
                report.rules.len(),
                failures,
                "{effort:?}/{policy}: {:?}",
                report.rules
            );
            assert_eq!(
                report.advisories.len(),
                advisories,
                "{effort:?}/{policy}: {:?}",
                report.advisories
            );
            for finding in report.rules.iter().chain(&report.advisories) {
                assert_eq!(finding.file, "query.mts");
                assert_eq!(finding.line, 5);
                assert!(
                    finding.message.contains("PoolClient")
                        && finding.message.contains("@example/db")
                        && finding.message.contains("executorTypeNames")
                );
            }
        }
    }
    for policy in ["report", "advise"] {
        let report = run_all_with_options(
            root.clone(),
            Some(format!("{policy}.yml").into()),
            None,
            true,
            false,
            AnalysisOptions::default(),
        )
        .unwrap();
        assert_eq!(
            report
                .suppressed
                .iter()
                .filter(|row| row.rule == "postgres-require-query-annotation")
                .count(),
            3
        );
    }
}

#[test]
fn imported_value_receivers_get_actionable_unknown_type_diagnostics_at_low_and_max() {
    use crate::check_runner::{run_all_with_options, AnalysisOptions};
    use crate::config::v2::Effort;

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/receiver-advice");
    for effort in [Effort::Max, Effort::Low] {
        for (policy, failures, advisories) in [("report", 2, 0), ("advise", 0, 2), ("ignore", 0, 0)]
        {
            let report = run_all_with_options(
                root.clone(),
                Some(format!("{policy}.yml").into()),
                None,
                true,
                false,
                AnalysisOptions {
                    effort: Some(effort),
                    include_sql_verdicts: false,
                },
            )
            .unwrap();
            assert_eq!(report.rules.len(), failures, "{effort:?}/{policy}");
            assert_eq!(report.advisories.len(), advisories, "{effort:?}/{policy}");
            let mut diagnostic_files = report
                .rules
                .iter()
                .chain(&report.advisories)
                .map(|finding| (finding.file.as_str(), finding.line))
                .collect::<Vec<_>>();
            diagnostic_files.sort();
            if policy == "ignore" {
                assert!(diagnostic_files.is_empty());
            } else {
                assert_eq!(
                    diagnostic_files,
                    [
                        ("packages/db/src/query.mts", 3),
                        ("packages/db/src/renamed.mts", 3)
                    ]
                );
            }
            for finding in report.rules.iter().chain(&report.advisories) {
                assert_eq!(finding.line, 3);
                let receiver = if finding.file.ends_with("renamed.mts") {
                    "renamedPool"
                } else {
                    "writePool"
                };
                assert!(finding.message.contains(&format!("receiver `{receiver}`")));
                assert!(finding.message.contains("no declared executor type"));
                assert!(finding.message.contains("executorTypeNames"));
                assert!(finding.message.contains("annotate its exported value"));
                assert!(!finding
                    .message
                    .contains("undeclared executor type `writePool`"));
            }
            if policy != "ignore" {
                assert_eq!(
                    report
                        .suppressed
                        .iter()
                        .filter(|row| row.rule == "postgres-require-query-annotation")
                        .count(),
                    1,
                    "{effort:?}/{policy}"
                );
            }
        }
    }

    let configured = run_all_with_options(
        root.clone(),
        Some("configured-value.yml".into()),
        None,
        false,
        false,
        AnalysisOptions {
            effort: Some(Effort::Max),
            include_sql_verdicts: false,
        },
    )
    .unwrap();
    assert_eq!(configured.rules.len(), 2);
    for finding in &configured.rules {
        assert!(finding.message.contains("no declared executor type"));
        assert!(finding.message.contains("annotate its exported value"));
        assert!(!finding
            .message
            .contains("configured executor receiver of type"));
        assert_eq!(
            finding.target.as_deref(),
            Some("unresolved-configured-executor-receiver")
        );
    }
}

#[test]
fn configured_member_receivers_remain_visible_in_opt_in_source_diagnostics() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/embedded/undeclared-member-receivers.ts"
    ));
    for resolve in [false, true] {
        for configured in [false, true] {
            let types = if configured {
                vec!["Executor", "PoolClient"]
            } else {
                vec!["Executor"]
            };
            let mut request = serde_json::json!({
                "sources": [{"fileName": "query.ts", "source": source}],
                "importSpecifier": "@example/db", "executorTypeNames": types,
                "resolveImports": resolve,
            });
            let legacy: serde_json::Value = serde_json::from_str(
                &extract_executed_postgres_sql_json_impl(request.clone()).unwrap(),
            )
            .unwrap();
            request["includeDiagnostics"] = true.into();
            let report: serde_json::Value =
                serde_json::from_str(&extract_executed_postgres_sql_json_impl(request).unwrap())
                    .unwrap();
            assert_eq!(report["statements"], legacy);
            assert_eq!(legacy.as_array().unwrap().len(), 1);
            let key = if configured {
                "unresolvedExecutorReceivers"
            } else {
                "undeclaredExecutorTypes"
            };
            assert_eq!(
                report[key],
                serde_json::json!([{"type":"PoolClient", "module":"@example/db", "count":9}]),
                "{resolve}/{configured}: {report}"
            );
            let other = if configured {
                "undeclaredExecutorTypes"
            } else {
                "unresolvedExecutorReceivers"
            };
            assert_eq!(report[other], serde_json::json!([]));
        }
    }
}
