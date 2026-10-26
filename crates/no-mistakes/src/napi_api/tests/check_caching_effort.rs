use super::*;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/no-nextjs-caching-effort/fixture")
}

#[test]
fn public_caching_effort_preserves_literal_findings_and_routes_residuals_once() {
    let root = root();
    for (config, expected_findings, expected_advisories) in [
        (".no-mistakes.max.yml", 11, 0),
        (".no-mistakes.low.yml", 11, 0),
        (".no-mistakes.low-advise.yml", 7, 4),
        (".no-mistakes.low-ignore.yml", 7, 0),
    ] {
        let report: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(
                    json!({ "root": root, "config": root.join(config) }).to_string(),
                ))
                .unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            expected_findings,
            "{config}: {report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            expected_advisories,
            "{config}: {report}"
        );
        assert_eq!(report["warnings"], json!([]), "{report}");
        // The indirect fetch on line 12 is suppressed in both diagnostic channels.
        assert!(
            report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .chain(report["advisories"].as_array().unwrap())
                .all(|row| row["file"] != "web/app/page.ts" || row["line"] != 12),
            "{report}"
        );
    }
}

#[test]
fn analyze_project_caching_low_matches_public_check_and_effort_override_wins() {
    let root = root();
    for (effort, expected_findings, expected_advisories) in [("low", 7, 4), ("max", 11, 0)] {
        let mut options = json!({ "root": root, "config": root.join(".no-mistakes.low-advise.yml"), "effort": effort });
        let check: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(
                &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
                    .unwrap(),
            )
            .unwrap(),
        );
        options.as_object_mut().unwrap().remove("effort");
        options["reports"] = json!([{ "type": "check", "effort": effort }]);
        let project: serde_json::Value = crate::test_support::without_hygiene_advisory_json(
            serde_json::from_str(
                &crate::napi_api::analyze_project::analyze_project_json_impl(
                    crate::napi_api::options::test_json_arg(options.to_string()),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(
            check["rules"].as_array().unwrap().len(),
            expected_findings,
            "{check}"
        );
        assert_eq!(
            check["advisories"].as_array().unwrap().len(),
            expected_advisories,
            "{check}"
        );
        assert_eq!(
            project["reports"][0]["result"]["rules"], check["rules"],
            "{project}"
        );
        assert_eq!(
            project["reports"][0]["result"]["advisories"], check["advisories"],
            "{project}"
        );
    }
}
