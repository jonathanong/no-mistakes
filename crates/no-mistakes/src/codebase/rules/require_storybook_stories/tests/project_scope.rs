use super::*;

fn scope_config(name: &str) -> NoMistakesConfig {
    let root = fixture("project-include-scope");
    crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap()
}

#[test]
fn generic_include_intersects_the_selected_project() {
    let root = fixture("project-include-scope");
    let findings = check(&root, &scope_config("generic-include.yml"), None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "web/components/Card.tsx");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("components/Card.tsx#Card")
    );
}

#[test]
fn component_include_selects_paths_relative_to_the_selected_project() {
    let root = fixture("project-include-scope");
    let findings = check(&root, &scope_config("options-include.yml"), None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "web/components/Card.tsx");
}

#[test]
fn component_include_remains_additive_with_export_kind_selection() {
    let root = fixture("project-include-scope");
    let findings = check(&root, &scope_config("additive-options-include.yml"), None).unwrap();
    let files = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<Vec<_>>();
    assert_eq!(files, ["web/components/Card.tsx", "web/other/Other.tsx"]);
}

#[test]
fn project_scope_matches_direct_aggregate_and_node_reports() {
    let root = fixture("project-include-scope");
    for config in [
        "generic-include.yml",
        "options-include.yml",
        "additive-options-include.yml",
    ] {
        let direct = check(&root, &scope_config(config), None).unwrap();
        let aggregate =
            crate::codebase::rules::run_check(&root, Some(&root.join(config)), None).unwrap();
        assert_eq!(direct, aggregate, "{config}");
        let report = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({"root": root, "config": root.join(config)}).to_string(),
        ))
        .unwrap();
        let report: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert_eq!(
            report["rules"],
            serde_json::to_value(&direct).unwrap(),
            "{config}: {report}"
        );
        assert_eq!(
            report["warnings"],
            serde_json::json!([]),
            "{config}: {report}"
        );
        assert_eq!(
            report["advisories"],
            serde_json::json!([]),
            "{config}: {report}"
        );
    }
}
