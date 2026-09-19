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
    assert!(
        value["reports"][0]["result"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["kind"] == "ambiguous-ownership"),
        "{value}"
    );
}

#[test]
fn exclusive_analyze_project_does_not_parse_excluded_tests() {
    let root = PathBuf::from(bounded_root());
    crate::ast::begin_parse_count(&root);
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
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert!(
        !counts.contains_key(&root.join("web/app/page.test.tsx")),
        "{counts:#?}"
    );
    assert!(
        !counts.contains_key(&root.join("web/lib/unrelated.test.ts")),
        "{counts:#?}"
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
