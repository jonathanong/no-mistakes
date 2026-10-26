use super::*;

#[test]
fn playwright_pattern_uncertainty_preserves_check_and_project_parity() {
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
        let fixture = crate::test_support::materialize_saved_fixture(&source);
        let root = fixture.path();
        let report: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root}).to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{name}: {report}");
        assert_eq!(findings[0]["rule"], "file-extension-policy");
        assert_eq!(findings[0]["file"], "forbidden.txt");
        let warnings = report["warnings"].as_array().unwrap();
        if let Some(property) = unknown {
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
        let project: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(
                    json!({"root": root, "reports": [{"type": "check"}]}).to_string(),
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(project["reports"][0]["result"], report, "{name}");
    }
}

#[test]
fn standalone_playwright_pattern_warning_survives_prepared_facts() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/playwright-configs/unknown-match/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let report: serde_json::Value = serde_json::from_str(
        &crate::napi_api::playwright_check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": fixture.path()}).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
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
fn batched_playwright_pattern_warnings_do_not_leak_to_a_known_sibling() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/playwright-configs/unknown-match/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let report: serde_json::Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(json!({
                "root": fixture.path(),
                "reports": [{"type": "check"}, {"type": "playwrightCheck", "project": "sibling"}]
            }).to_string()),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!report["reports"][0]["result"]["warnings"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(
        report["reports"][1]["result"].get("warnings").is_none(),
        "{report}"
    );
}

#[test]
fn playwright_pattern_integration_policies_warn_only_for_selected_unknown_projects() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/playwright-configs/unknown-match/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    for (config, uncertain) in [
        ("integration-only.yml", true),
        ("known-integration.yml", false),
    ] {
        let report: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": fixture.path(), "config": config}).to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            1,
            "{config}: {report}"
        );
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

#[test]
fn standalone_integration_pattern_uncertainty_is_not_a_clean_report() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/playwright-configs/unknown-match/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let error = crate::integration_tests::check(
        fixture.path(),
        Some(&fixture.path().join("integration-only.yml")),
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("playwright.config.ts:5"),
        "{error}"
    );
}
