use super::*;
use serde_json::json;

#[path = "scope_import_bounds_tests.rs"]
mod scope_import_bounds_tests;
#[path = "scope_input_error_tests.rs"]
mod scope_input_error_tests;
#[path = "seed_boundary_tests.rs"]
mod seed_boundary_tests;
#[path = "target_boundary_tests.rs"]
mod target_boundary_tests;

fn options(reports: Value) -> AnalyzeProjectOptions {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/simple/fixture");
    serde_json::from_value(json!({ "root": root, "reports": reports })).unwrap()
}

#[test]
fn prepared_call_sites_reject_blank_targets_and_files_outside_prepared_facts() {
    let prepared = options(json!([{"type": "symbols", "files": ["a.mts"]}]));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    for (report, message) in [
        (
            json!({"type":"callSites", "file":" ", "exportName":"a"}),
            "file is required",
        ),
        (
            json!({"type":"callSites", "file":"a.mts", "exportName":" "}),
            "exportName is required",
        ),
        (
            json!({"type":"callSites", "file":"absent.mts", "exportName":"a"}),
            "missing facts for",
        ),
    ] {
        let request = serde_json::from_value(report).unwrap();
        let error = scope.call_sites_report(&request, &prepared).unwrap_err();
        assert!(error.to_string().contains(message), "{error:#}");
    }
}

#[test]
fn check_scope_paths_inherit_from_master_root_and_override_from_report_root() {
    let mut prepared = options(json!([
        {"type":"check", "root":"child", "effort":"low"},
        {"type":"symbols", "root":"child", "tsconfig":"local.json", "config":"local.yml"}
    ]));
    prepared.tsconfig = Some("master.json".into());
    prepared.config = Some("master.yml".into());
    let inherited = effective_scope(&prepared.reports[0], &prepared).unwrap();
    let master = PathBuf::from(prepared.root.as_ref().unwrap());
    assert_eq!(
        inherited.tsconfig,
        Some(crate::codebase::ts_resolver::normalize_path(
            &master.join("master.json")
        ))
    );
    assert_eq!(
        inherited.config,
        Some(crate::codebase::ts_resolver::normalize_path(
            &master.join("master.yml")
        ))
    );
    assert!(!inherited.automatic_tsconfig);
    let overridden = effective_scope(&prepared.reports[1], &prepared).unwrap();
    assert_eq!(
        overridden.tsconfig,
        Some(overridden.root.join("local.json"))
    );
    assert_eq!(overridden.config, Some(overridden.root.join("local.yml")));
    assert_eq!(overridden.key.effort, None);
}

#[test]
fn import_seed_plan_unions_duplicate_bounds_and_marks_other_graph_consumers() {
    let prepared = options(json!([
        {"type":"dependencies", "files":["a.mts"], "relationships":["import"]},
        {"type":"dependencies", "files":["b.mts"], "relationships":["import"]},
        {"type":"resolveCheckDependencies", "files":["a.mts"]}
    ]));
    let plan = classify_import_seed_requests(&prepared).unwrap();
    assert!(!plan.other_graph_consumer);
    assert_eq!(plan.unbounded.unwrap().files.len(), 2);
    assert!(plan.bounded.is_empty());

    for report in [
        json!({"type":"dependents", "files":["a.mts"], "relationships":["import"]}),
        json!({"type":"dependencies", "files":["a.mts"], "relationships":["call"]}),
        json!({"type":"symbols", "files":["a.mts"]}),
    ] {
        let prepared = options(json!([report]));
        let plan = classify_import_seed_requests(&prepared).unwrap();
        assert!(plan.other_graph_consumer);
        assert!(plan.unbounded.is_none());
        assert!(plan.exclusive_bounds().is_none());
    }
}

#[test]
fn scope_validation_reports_bad_types_for_each_inherited_path() {
    for key in ["root", "tsconfig", "config", "effort"] {
        let mut report = json!({"type":"check"});
        report[key] = json!(42);
        let prepared = options(json!([report]));
        let error = effective_scope(&prepared.reports[0], &prepared)
            .err()
            .unwrap();
        assert_eq!(error.to_string(), format!("{key} must be a string"));
    }
}
