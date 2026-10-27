use super::*;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/postgres-parse-failure-locations/fixture"),
    )
}

fn check_report(root: &std::path::Path, config: &str, effort: &str) -> serde_json::Value {
    crate::test_support::without_hygiene_advisory_json(
        serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": root.join(config), "effort": effort}).to_string(),
            ))
            .unwrap(),
        )
        .unwrap(),
    )
}

fn rust_analysis_warnings(root: &std::path::Path, config: &str) -> Vec<String> {
    let mut config = crate::config::v2::load_v2_config(root, Some(&root.join(config))).unwrap();
    config.effort_override = Some(crate::config::v2::Effort::Low);
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        root,
        &mut config,
        &[],
        &sources,
    )
    .unwrap();
    config.analysis_warnings
}

fn uuid_locations(report: &serde_json::Value, channel: &str) -> Vec<(String, u64)> {
    report[channel]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["rule"] == "postgres-no-transformed-uuid-predicates")
        .map(|row| {
            (
                row["file"].as_str().unwrap().into(),
                row["line"].as_u64().unwrap(),
            )
        })
        .collect()
}

fn idempotent_locations(report: &serde_json::Value, is_schema_finding: bool) -> Vec<(String, u64)> {
    report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["rule"] == "postgres-idempotent-insert"
                && row["message"].as_str().is_some_and(|message| {
                    message.contains("could not be analyzed for postgres-idempotent-insert")
                        == is_schema_finding
                })
        })
        .map(|row| {
            (
                row["file"].as_str().unwrap().into(),
                row["line"].as_u64().unwrap(),
            )
        })
        .collect()
}

#[test]
fn parse_failure_locations_public_check_preserves_parse_findings_and_independent_suppression() {
    let root = root();
    for effort in ["low", "max"] {
        for (config, uuid_file, uuid_lines, schema_file, schema_lines, count) in [
            (
                ".no-mistakes.failures.yml",
                "failures.sql",
                vec![3, 6],
                "failures.sql",
                vec![3, 6],
                16,
            ),
            (
                ".no-mistakes.one-suppressed.yml",
                "one-suppressed.sql",
                vec![6],
                "one-suppressed-idempotent-insert.sql",
                vec![6],
                8,
            ),
            (
                ".no-mistakes.suppressed.yml",
                "suppressed.sql",
                vec![],
                "suppressed-idempotent-insert.sql",
                vec![],
                0,
            ),
        ] {
            let report = check_report(&root, config, effort);
            let expected: Vec<(String, u64)> = uuid_lines
                .into_iter()
                .map(|line| (uuid_file.into(), line))
                .collect();
            assert_eq!(
                uuid_locations(&report, "rules"),
                expected,
                "{effort}/{config}: {report}"
            );
            assert_eq!(report["rules"].as_array().unwrap().len(), count);
            let expected_schema: Vec<(String, u64)> = schema_lines
                .into_iter()
                .map(|line| (schema_file.into(), line))
                .collect();
            assert_eq!(
                idempotent_locations(&report, true),
                expected_schema,
                "{effort}/{config}: {report}"
            );
            assert!(idempotent_locations(&report, false).is_empty(), "{report}");
            assert_eq!(report["advisories"], json!([]));
            let warnings = report["warnings"].as_array().unwrap();
            if effort == "low" {
                // Compare the public API with registry-backed Rust policy metadata,
                // so new low adopters do not require another warning-count edit.
                assert!(!warnings.is_empty(), "{report}");
                assert_eq!(
                    report["warnings"],
                    json!(rust_analysis_warnings(&root, config)),
                    "{report}"
                );
                assert!(warnings.iter().all(|warning| {
                    let message = warning.as_str().unwrap();
                    message.contains("requested effort low")
                        && message.contains("effective effort max")
                        && !message.contains("postgres-no-transformed-uuid-predicates")
                }));
            } else {
                assert!(warnings.is_empty(), "{report}");
            }
        }
    }
}

#[test]
fn parse_failure_locations_public_advice_matches_analyze_project() {
    let root = root();
    for effort in ["low", "max"] {
        for (config, file, lines) in [
            (".no-mistakes.uuid-advice.yml", "failures.sql", vec![3, 6]),
            (
                ".no-mistakes.uuid-one-suppressed-advice.yml",
                "one-suppressed.sql",
                vec![6],
            ),
        ] {
            let check = check_report(&root, config, effort);
            let expected: Vec<(String, u64)> =
                lines.into_iter().map(|line| (file.into(), line)).collect();
            assert_eq!(
                uuid_locations(&check, "advisories"),
                expected,
                "{effort}/{config}: {check}"
            );
            assert_eq!(check["rules"], json!([]));
            assert_eq!(check["warnings"], json!([]));
            let project: serde_json::Value = crate::test_support::without_hygiene_advisory_json(serde_json::from_str(
                &crate::napi_api::analyze_project::analyze_project_json_impl(
                    crate::napi_api::options::test_json_arg(
                        json!({"root": root, "config": root.join(config), "reports": [{"type": "check", "effort": effort}]}).to_string(),
                    ),
                ).unwrap(),
            ).unwrap());
            assert_eq!(project["reports"][0]["result"], check);
        }
    }
}
