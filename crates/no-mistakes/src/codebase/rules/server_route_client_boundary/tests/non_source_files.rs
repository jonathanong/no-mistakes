use super::*;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::ts_source::facts::TsFactPlan;

#[test]
fn metadata_import_placeholders_do_not_abort_source_boundary_checks() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("non-source-project-files"));
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let paths =
        crate::codebase::rules::production_dependency_declarations::required_import_fact_files(
            &root, &config, &files,
        )
        .unwrap();
    let shared = collect_check_facts(
        &root,
        files,
        CheckFactPlan {
            server_route_client_boundary: true,
            graph: TsFactPlan::imports(),
            production_import_paths: paths,
            ..Default::default()
        },
    );
    // Exercise real metadata placeholders rather than absent fact-map entries.
    for name in ["AGENTS.md", "metadata.json", "package.json"] {
        let metadata = shared.ts.get(&root.join("backend").join(name)).unwrap();
        assert!(metadata.production_imports.is_some());
        assert!(metadata.server_route_client_boundary.is_none());
    }
    let findings = check_with_facts(&root, &config, &shared).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "backend/api/client.ts");
    assert_eq!(findings[0].line, 5);
}

#[test]
fn aggregate_report_keeps_source_violation_with_project_metadata() {
    let root = fixture("non-source-project-files");
    let report = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({"root": root}).to_string(),
    ))
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&report).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(findings[0]["rule"], RULE_ID);
    assert_eq!(findings[0]["file"], "backend/api/client.ts");
    assert_eq!(findings[0]["line"], 5);
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert!(!report.to_string().contains("requires boundary facts"));
}
