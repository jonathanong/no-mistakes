use super::*;

#[test]
fn empty_dependency_roots_do_not_build_import_graphs() {
    for bounded in [false, true] {
        let mut report = json!({"type":"dependencies", "files":[], "relationships":["import"]});
        if bounded {
            report["candidateInclude"] = json!(["*.mts"]);
        }
        let prepared = options(json!([report]));
        let plan = classify_import_seed_requests(&prepared).unwrap();
        if bounded {
            assert!(plan.exclusive_bounds().unwrap().files.is_empty());
        } else {
            assert!(plan.unbounded.unwrap().files.is_empty());
        }
        let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
        let scope = context.scopes.values().next().unwrap();
        assert_eq!(scope.traversal.graph_build_count(), 0);
        assert_eq!(scope.traversal.candidate_inventory_applied(), bounded);
    }
}

#[test]
fn import_seed_and_prewarm_reject_invalid_requests_before_building_graphs() {
    let prepared = options(json!([{"type":"symbols", "files":["a.mts"]}]));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    for report in [
        json!({"type":"dependents", "files":42}),
        json!({"type":"dependencies", "files":42}),
        json!({"type":"dependencies", "files":["a.mts"], "candidateInclude":["*.mts"], "includeSymbols":true}),
    ] {
        let invalid = options(json!([report]));
        assert!(classify_import_seed_requests(&invalid).is_err());
        assert!(scope.prewarm_graph_for_report(&invalid.reports[0]).is_err());
    }
    assert_eq!(scope.traversal.graph_build_count(), 0);
}

#[test]
fn scope_preparation_propagates_domain_configuration_errors() {
    for (report, expected) in [
        (
            json!({"type":"importUsages", "filters":false}),
            "invalid type",
        ),
        (
            json!({"type":"callSites", "file":false, "exportName":"a"}),
            "invalid type",
        ),
        (json!({"type":"flow", "target":false}), "invalid type"),
        (
            json!({"type":"flow", "target":"a.mts#a", "direction":"invalid"}),
            "unknown flow direction",
        ),
        (
            json!({"type":"check", "config":"absent-config.yml"}),
            "absent-config.yml",
        ),
        (
            json!({"type":"symbols", "files":["a.mts"], "tsconfig":"absent-tsconfig.json"}),
            "absent-tsconfig.json",
        ),
        (
            json!({"type":"check", "includeRunnerConfigDeadlines":"invalid"}),
            "invalid type",
        ),
        (
            json!({"type":"effects", "kind":"invalid", "entry":"a.mts"}),
            "invalid",
        ),
        (
            json!({"type":"effects", "kind":"fetch", "categories":false}),
            "invalid type",
        ),
        (
            json!({"type":"playwrightCheck", "files":false}),
            "invalid type",
        ),
        (
            json!({"type":"playwrightCheck", "playwrightConfig":["absent.config.ts"]}),
            "absent.config.ts",
        ),
    ] {
        let prepared = options(json!([report]));
        let error = AnalyzeProjectContext::prepare(&prepared)
            .err()
            .expect("invalid preparation");
        assert!(format!("{error:#}").contains(expected), "{error:#}");
    }
}

#[test]
fn check_planning_rejects_malformed_deadline_and_effort_values() {
    for (key, value) in [
        ("includeRunnerConfigDeadlines", json!("invalid")),
        ("effort", json!(false)),
    ] {
        let mut report = json!({"type":"check"});
        report[key] = value;
        let prepared = options(json!([report]));
        assert!(requested_deadline_evidence(&prepared).is_err());
        assert!(requested_check_effort(&prepared).is_err());
    }
}

#[test]
fn derived_dependency_requests_validate_ids_before_collecting_closures() {
    let prepared = options(json!([
        {"type":"dependencies", "id":"a", "files":["a.mts"], "relationships":["import"]},
        {"type":"dependencies", "id":"ambiguous", "files":["a.mts"], "relationships":["import"]},
        {"type":"dependencies", "id":"ambiguous", "files":["b.mts"], "relationships":["import"]}
    ]));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    for (report, expected) in [
        (json!({}), "dependencyReportIds is required"),
        (json!({"dependencyReportIds":false}), "must be an array"),
        (json!({"dependencyReportIds":[]}), "at least one ID"),
        (json!({"dependencyReportIds":[false]}), "non-empty strings"),
        (json!({"dependencyReportIds":[""]}), "non-empty strings"),
        (
            json!({"dependencyReportIds":["missing"]}),
            "references no dependencies report",
        ),
        (json!({"dependencyReportIds":["a","a"]}), "duplicate ID"),
        (json!({"dependencyReportIds":["ambiguous"]}), "ambiguous"),
    ] {
        let mut report = report;
        report["type"] = json!("resolveCheckDependencies");
        let request = serde_json::from_value(report).unwrap();
        let error = scope
            .resolve_check_dependencies_report(&request)
            .unwrap_err();
        assert!(error.to_string().contains(expected), "{error:#}");
    }
}
