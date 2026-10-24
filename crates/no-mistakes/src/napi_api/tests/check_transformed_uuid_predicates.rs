use super::*;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/postgres-no-transformed-uuid-predicates/fixture"),
    )
}

#[test]
fn transformed_uuid_predicates_public_check_routes_both_channels() {
    let root = root();
    for (config, findings, advisories) in [
        (".no-mistakes.yml", 11, 0),
        (".no-mistakes.max.yml", 11, 0),
        (".no-mistakes.low-report.yml", 11, 0),
        (".no-mistakes.low-advise.yml", 10, 1),
        (".no-mistakes.low-ignore.yml", 10, 0),
    ] {
        let report: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": root.join(config)}).to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
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
        assert_eq!(report["warnings"], json!([]));
    }
}

#[test]
fn transformed_uuid_predicates_analyze_project_matches_check_effort_override() {
    let root = root();
    for effort in ["low", "max"] {
        let config = root.join(".no-mistakes.low-advise.yml");
        let check: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": config, "effort": effort}).to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        let project: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({"root": root, "config": config, "reports": [{"type": "check", "effort": effort}]}).to_string(),
                ),
            ).unwrap(),
        ).unwrap();
        assert_eq!(project["reports"][0]["result"], check);
    }
}

#[test]
fn transformed_uuid_predicates_public_raw_sql_advice_does_not_fail() {
    let root = root();
    for effort in ["low", "max"] {
        let report: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": root.join(".no-mistakes.raw-advice.yml"), "effort": effort}).to_string(),
            )).unwrap(),
        ).unwrap();
        assert_eq!(report["rules"], json!([]));
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            2,
            "{report}"
        );
        assert_eq!(report["warnings"], json!([]));
    }
}
