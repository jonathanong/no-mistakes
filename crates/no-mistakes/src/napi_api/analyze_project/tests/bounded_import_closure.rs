use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn simple_root() -> String {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/simple/fixture"),
    )
    .display()
    .to_string()
}

fn bounded_root() -> String {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/bounded-import-closure"),
    )
    .display()
    .to_string()
}

#[test]
fn additive_bounded_report_does_not_change_unbounded_deps_fields() {
    let root = simple_root();
    let baseline = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [{
                "type": "dependencies",
                "id": "open",
                "files": ["a.mts"],
                "relationships": ["import-static", "import-dynamic", "import-type"]
            }]
        })
        .to_string(),
    ))
    .unwrap();
    let mixed = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "open",
                    "files": ["a.mts"],
                    "relationships": ["import-static", "import-dynamic", "import-type"]
                },
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["a.mts"],
                    "relationships": ["import-static"],
                    "candidateInclude": ["**/*"],
                    "projection": "paths"
                }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let baseline: Value = serde_json::from_str(&baseline).unwrap();
    let mixed: Value = serde_json::from_str(&mixed).unwrap();
    assert_eq!(
        mixed["reports"][0]["result"],
        baseline["reports"][0]["result"]
    );
}

#[test]
fn additive_bounded_report_does_not_change_dependents_fields() {
    let root = simple_root();
    let baseline = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [{ "type": "dependents", "id": "deps", "files": ["b.mts"] }]
        })
        .to_string(),
    ))
    .unwrap();
    let mixed = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                { "type": "dependents", "id": "deps", "files": ["b.mts"] },
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["a.mts"],
                    "relationships": ["import-static"],
                    "candidateInclude": ["**/*"],
                    "projection": "paths"
                }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let baseline: Value = serde_json::from_str(&baseline).unwrap();
    let mixed: Value = serde_json::from_str(&mixed).unwrap();
    assert_eq!(
        mixed["reports"][0]["result"],
        baseline["reports"][0]["result"]
    );
}

#[test]
fn analyze_project_paths_projection_matches_standalone() {
    let root = bounded_root();
    let request = json!({
        "files": ["web/app/page.tsx"],
        "relationships": ["import-static", "import-dynamic", "import-type"],
        "candidateInclude": ["web/**"],
        "candidateExclude": ["**/*.test.*"],
        "projection": "paths"
    });
    let standalone =
        crate::napi_api::codebase::dependencies_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "files": request["files"],
                "relationships": request["relationships"],
                "candidateInclude": request["candidateInclude"],
                "candidateExclude": request["candidateExclude"],
                "projection": "paths"
            })
            .to_string(),
        ))
        .unwrap();
    let batched = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [{
                "type": "dependencies",
                "id": "closure",
                "files": request["files"],
                "relationships": request["relationships"],
                "candidateInclude": request["candidateInclude"],
                "candidateExclude": request["candidateExclude"],
                "projection": "paths"
            }]
        })
        .to_string(),
    ))
    .unwrap();
    let standalone: Value = serde_json::from_str(&standalone).unwrap();
    let batched: Value = serde_json::from_str(&batched).unwrap();
    assert_eq!(batched["reports"][0]["result"], standalone);
}

#[test]
fn derived_resolve_check_reuses_dependency_closure() {
    let root = simple_root();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["a.mts"],
                    "relationships": ["import-static", "import-dynamic", "import-type", "workspace"],
                    "projection": "paths"
                },
                {
                    "type": "resolveCheckDependencies",
                    "dependencyReportIds": ["closure"]
                }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let derived = &value["reports"][1]["result"];
    assert_eq!(derived["allResolve"], true, "{derived}");
    assert_eq!(derived["results"].as_array().unwrap().len(), 3, "{derived}");
    assert!(derived["results"]
        .as_array()
        .unwrap()
        .iter()
        .any(|result| result["file"] == "c.mts"));
}

#[test]
fn derived_resolve_check_rejects_unknown_dependency_report_id() {
    let error = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": simple_root(),
            "reports": [{
                "type": "resolveCheckDependencies",
                "dependencyReportIds": ["missing"]
            }]
        })
        .to_string(),
    ))
    .unwrap_err();
    assert!(
        error
            .reason
            .contains("no dependencies report with id `missing`"),
        "{error}"
    );
}

#[test]
fn derived_resolve_check_rejects_non_import_dependency_report() {
    let error = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": simple_root(),
            "reports": [
                {
                    "type": "dependencies",
                    "id": "mixed",
                    "files": ["a.mts"],
                    "relationships": ["call"]
                },
                { "type": "resolveCheckDependencies", "dependencyReportIds": ["mixed"] }
            ]
        })
        .to_string(),
    ))
    .unwrap_err();
    assert!(
        error
            .reason
            .contains("must use import or workspace relationships"),
        "{error}"
    );
}

#[test]
fn derived_resolve_check_matches_standalone_for_local_alias_and_external_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries/fixture"),
    )
    .display()
    .to_string();
    let standalone =
        crate::napi_api::queries::resolve_check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "files": ["broken.ts"] }).to_string(),
        ))
        .unwrap();
    let derived = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["broken.ts"],
                    "relationships": ["import-static", "import-dynamic", "import-type"]
                },
                { "type": "resolveCheckDependencies", "dependencyReportIds": ["closure"] }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let standalone: Value = serde_json::from_str(&standalone).unwrap();
    let derived: Value = serde_json::from_str(&derived).unwrap();
    assert_eq!(
        derived["reports"][1]["result"]["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|result| result["file"] == "broken.ts")
            .unwrap(),
        &standalone["results"][0]
    );
}

#[test]
fn derived_resolve_check_keeps_computed_imports_unresolved() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries-kinds/fixture"),
    )
    .display()
    .to_string();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["computed.ts"],
                    "relationships": ["import-static", "import-dynamic", "import-type"]
                },
                { "type": "resolveCheckDependencies", "dependencyReportIds": ["closure"] }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let result = &value["reports"][1]["result"]["results"][0];
    assert_eq!(result["allResolve"], false, "{result}");
    assert!(result["imports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["computed"] == true && row["status"] == "unresolved"));
}

#[test]
fn derived_resolve_check_falls_back_for_an_automatic_invalid_tsconfig() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis/query-invalid-tsconfig"),
    )
    .display()
    .to_string();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({"root": root, "reports": [
            {"type":"dependencies","id":"closure","files":["entry.ts"],"relationships":["import-static"]},
            {"type":"resolveCheckDependencies","dependencyReportIds":["closure"]}
        ]}).to_string(),
    ))
    .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&output).unwrap()["reports"][1]["result"]["allResolve"],
        true
    );
}

#[test]
fn derived_resolve_check_honors_an_explicit_tsconfig() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/queries/nested-target-alias"),
    );
    let request = json!({"root": root, "tsconfig":"tsconfig.json", "reports": [
        {"type":"dependencies","id":"closure","files":["nested/src/alias-consumer.ts"],"relationships":["import-static"]},
        {"type":"resolveCheckDependencies","dependencyReportIds":["closure"]}
    ]});
    let derived: Value = serde_json::from_str(
        &analyze_project_json_impl(crate::napi_api::options::test_json_arg(request.to_string()))
            .unwrap(),
    )
    .unwrap();
    let standalone: Value = serde_json::from_str(&crate::napi_api::queries::resolve_check_json_impl(crate::napi_api::options::test_json_arg(json!({"root": root, "tsconfig":"tsconfig.json", "files":["nested/src/alias-consumer.ts"]}).to_string())).unwrap()).unwrap();
    assert_eq!(
        derived["reports"][1]["result"]["results"][0],
        standalone["results"][0]
    );
    assert_eq!(standalone["results"][0]["imports"][0]["status"], "external");
}

#[test]
fn analyze_project_graph_projection_matches_standalone() {
    let root = bounded_root();
    let standalone =
        crate::napi_api::codebase::dependencies_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "files": ["web/app/page.tsx"],
                "relationships": ["import-static", "import-dynamic", "import-type"],
                "candidateInclude": ["web/**"],
                "candidateExclude": ["**/*.test.*"]
            })
            .to_string(),
        ))
        .unwrap();
    let batched = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [{
                "type": "dependencies",
                "id": "closure",
                "files": ["web/app/page.tsx"],
                "relationships": ["import-static", "import-dynamic", "import-type"],
                "candidateInclude": ["web/**"],
                "candidateExclude": ["**/*.test.*"]
            }]
        })
        .to_string(),
    ))
    .unwrap();
    let standalone: Value = serde_json::from_str(&standalone).unwrap();
    let batched: Value = serde_json::from_str(&batched).unwrap();
    assert_eq!(batched["reports"][0]["result"], standalone);
    assert!(standalone.get("roots").is_some());
}

#[test]
fn dependents_report_rejects_candidate_include() {
    let request = json!({
        "root": simple_root(),
        "reports": [{
            "type": "dependents",
            "files": ["b.mts"],
            "candidateInclude": ["**/*"]
        }]
    });
    let options: super::types::AnalyzeProjectOptions =
        serde_json::from_value(request.clone()).unwrap();
    assert_eq!(
        options.reports[0]
            .options
            .get("candidateInclude")
            .and_then(|value| value.as_array())
            .map(|values| values.len()),
        Some(1),
        "{:#?}",
        options.reports[0].options
    );
    let args = super::traverse_args(&options.reports[0], &options).unwrap();
    assert!(
        args.has_candidate_bounds(),
        "candidate_include={:?} candidate_exclude={:?}",
        args.candidate_include,
        args.candidate_exclude
    );
    let err =
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(request.to_string()))
            .unwrap_err();
    assert!(format!("{err}").contains("dependencies"), "{err}");
}

#[test]
fn exclusive_analyze_project_keeps_seed_diagnostics() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/workspace-resolution"),
    );
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [{
                "type": "dependencies",
                "id": "closure",
                "files": ["apps/ambiguous/src/entry.ts"],
                "relationships": ["import-static", "import-dynamic", "import-type"],
                "candidateInclude": ["**/*"],
                "projection": "paths"
            }]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let ambiguous = value["reports"][0]["result"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|diagnostic| diagnostic["kind"] == "ambiguous-ownership")
        .count();
    assert_eq!(ambiguous, 1, "{value}");
}

#[test]
fn shared_bounds_do_not_replay_other_report_seed_diagnostics() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/workspace-resolution"),
    );
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "ambiguous",
                    "files": ["apps/ambiguous/src/entry.ts"],
                    "relationships": ["import-static", "import-dynamic", "import-type"],
                    "candidateInclude": ["**/*"],
                    "projection": "paths"
                },
                {
                    "type": "dependencies",
                    "id": "web",
                    "files": ["apps/web/src/entry.ts"],
                    "relationships": ["import-static", "import-dynamic", "import-type"],
                    "candidateInclude": ["**/*"],
                    "projection": "paths"
                }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let reports = value["reports"].as_array().unwrap();
    let ambiguous = reports
        .iter()
        .find(|report| report["id"] == "ambiguous")
        .unwrap();
    let web = reports.iter().find(|report| report["id"] == "web").unwrap();
    assert!(
        ambiguous["result"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["kind"] == "ambiguous-ownership"),
        "{value}"
    );
    assert!(
        web["result"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|diagnostic| diagnostic["kind"] != "ambiguous-ownership"),
        "{value}"
    );
}

#[test]
fn exclusive_analyze_project_honors_finite_depth() {
    let root = bounded_root();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "reports": [{
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["web/app/page.tsx"],
                    "relationships": ["import-static", "import-dynamic", "import-type"],
                    "candidateInclude": ["web/**"],
                    "candidateExclude": ["**/*.test.*"],
                    "depth": 0,
                    "projection": "paths"
                }]
            })
            .to_string(),
        ))
        .unwrap()
    };
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][0]["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry.as_str())
        .collect::<Vec<_>>();
    assert!(
        !files.iter().any(|path| path.contains("packages/ui")),
        "{files:?}"
    );
    let work = observer.snapshot().work;
    assert!(
        work.get("parse.files").copied().unwrap_or(0) <= 1,
        "{work:#?}"
    );
}

#[test]
fn exclusive_analyze_project_does_not_parse_excluded_tests() {
    let root = bounded_root();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "reports": [{
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["web/app/page.tsx"],
                    "relationships": ["import-static", "import-dynamic", "import-type"],
                    "candidateInclude": ["web/**"],
                    "candidateExclude": ["**/*.test.*"],
                    "projection": "paths"
                }]
            })
            .to_string(),
        ))
        .unwrap()
    };
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"][0]["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry.as_str())
        .collect::<Vec<_>>();
    assert!(
        !files.iter().any(|path| path.contains("page.test")),
        "{files:?}"
    );
    assert!(
        !files.iter().any(|path| path.contains("unrelated.test")),
        "{files:?}"
    );
    let work = observer.snapshot().work;
    assert!(work["graph.candidate_excluded"] >= 2, "{work:#?}");
    assert!(
        work["parse.files"] >= 1 && work["parse.files"] <= 4,
        "{work:#?}"
    );
}

#[test]
fn related_report_rejects_candidate_exclude() {
    let err = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": simple_root(),
            "reports": [{
                "type": "related",
                "files": ["b.mts"],
                "candidateExclude": ["**/*.test.*"]
            }]
        })
        .to_string(),
    ))
    .unwrap_err();
    assert!(format!("{err}").contains("dependencies"), "{err}");
}
