use super::*;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/check-runner/finite-set-effort/fixture"),
    )
}

#[test]
fn public_finite_set_effort_routes_findings_and_advisories() {
    let root = root();
    for (config, findings, advisories) in [
        ("default.yml", 4, 0),
        ("max.yml", 4, 0),
        ("low-report.yml", 4, 0),
        ("low-advise.yml", 1, 3),
        ("low-ignore.yml", 1, 0),
        ("mixed.yml", 3, 1),
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
fn analyze_project_finite_set_effort_matches_check() {
    let root = root();
    for (effort, findings, advisories) in [("low", 1, 3), ("max", 4, 0)] {
        let check: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": root.join("low-advise.yml"), "effort": effort})
                    .to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        let project: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({"root": root, "config": root.join("low-advise.yml"), "reports": [{"type": "check", "effort": effort}]}).to_string(),
                ),
            ).unwrap(),
        ).unwrap();
        assert_eq!(check["rules"].as_array().unwrap().len(), findings);
        assert_eq!(check["advisories"].as_array().unwrap().len(), advisories);
        assert_eq!(project["reports"][0]["result"], check);
    }
}
